use crate::{RecordData, RecordType, Ttl, ZoneName};
use dns_update::{DnsUpdater, Error};

#[derive(Clone, Copy)]
pub(crate) enum ProviderKind {
    Cloudflare,
    Cloudns,
}

/// An asynchronous DNS client scoped to one provider and zone.
pub struct DnsProvider {
    pub(crate) updater: DnsUpdater,
    pub(crate) zone: ZoneName,
    pub(crate) kind: ProviderKind,
}

impl DnsProvider {
    pub(crate) fn check_name(&self, name: &str) -> Result<(), Error> {
        let name = name.trim_end_matches('.').to_ascii_lowercase();
        if name == self.zone.as_str()
            || name
                .strip_suffix(self.zone.as_str())
                .is_some_and(|p| p.ends_with('.'))
        {
            Ok(())
        } else {
            Err(Error::BadRequest)
        }
    }

    pub async fn list(&self, name: &str, kind: RecordType) -> Result<Vec<RecordData>, Error> {
        self.check_name(name)?;
        self.updater
            .list_rrset(name, kind, self.zone.as_str())
            .await
    }

    pub async fn add(&self, name: &str, ttl: Ttl, data: RecordData) -> Result<(), Error> {
        let kind = record_type(&data);
        self.add_many(name, kind, ttl, vec![data]).await
    }

    /// Add values to one RRset. All values must match `kind`. Provider writes
    /// may partially succeed before an error; concurrent changes are not atomic.
    pub async fn add_many(
        &self,
        name: &str,
        kind: RecordType,
        ttl: Ttl,
        mut records: Vec<RecordData>,
    ) -> Result<(), Error> {
        self.check_name(name)?;
        match self.kind {
            ProviderKind::Cloudflare => ttl.validate_cloudflare(),
            ProviderKind::Cloudns => ttl.validate_cloudns(),
        }
        .map_err(|_| Error::BadRequest)?;
        validate_records(kind, &records)?;
        deduplicate(&mut records);
        if records.is_empty() {
            return Ok(());
        }
        self.updater
            .add_to_rrset(
                name,
                kind,
                ttl.as_provider_value(),
                records,
                self.zone.as_str(),
            )
            .await
    }

    /// Remove one matching value, leaving other values in the RRset intact.
    pub async fn remove(&self, name: &str, data: RecordData) -> Result<(), Error> {
        let kind = record_type(&data);
        self.remove_many(name, kind, vec![data]).await
    }

    /// Remove matching values from one RRset, preserving other values. All
    /// values must match `kind`. Writes can partially succeed and are not atomic.
    pub async fn remove_many(
        &self,
        name: &str,
        kind: RecordType,
        mut records: Vec<RecordData>,
    ) -> Result<(), Error> {
        self.check_name(name)?;
        validate_records(kind, &records)?;
        deduplicate(&mut records);
        if records.is_empty() {
            return Ok(());
        }
        self.updater
            .remove_from_rrset(name, kind, records, self.zone.as_str())
            .await
    }
}

fn record_type(data: &RecordData) -> RecordType {
    match data {
        RecordData::A(_) => RecordType::A,
        RecordData::AAAA(_) => RecordType::AAAA,
        RecordData::CNAME(_) => RecordType::CNAME,
        RecordData::NS(_) => RecordType::NS,
        RecordData::MX(_) => RecordType::MX,
        RecordData::TXT(_) => RecordType::TXT,
        RecordData::SRV(_) => RecordType::SRV,
        RecordData::TLSA(_) => RecordType::TLSA,
        RecordData::CAA(_) => RecordType::CAA,
    }
}

fn validate_records(kind: RecordType, records: &[RecordData]) -> Result<(), Error> {
    if records.iter().all(|record| record_type(record) == kind) {
        Ok(())
    } else {
        Err(Error::BadRequest)
    }
}

fn deduplicate(records: &mut Vec<RecordData>) {
    let mut seen = std::collections::HashSet::new();
    records.retain(|record| seen.insert(record.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_values_are_checked_and_deduplicated() {
        let a = RecordData::A("192.0.2.1".parse().unwrap());
        let other = RecordData::A("192.0.2.2".parse().unwrap());
        let mut records = vec![a.clone(), other.clone(), a.clone()];
        assert!(validate_records(RecordType::A, &records).is_ok());
        assert!(validate_records(RecordType::TXT, &records).is_err());
        deduplicate(&mut records);
        assert_eq!(records, vec![a, other]);
    }
}
