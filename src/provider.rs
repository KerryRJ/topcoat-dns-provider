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
        self.check_name(name)?;
        match self.kind {
            ProviderKind::Cloudflare => ttl.validate_cloudflare(),
            ProviderKind::Cloudns => ttl.validate_cloudns(),
        }
        .map_err(|_| Error::BadRequest)?;
        self.updater
            .add_to_rrset(
                name,
                match &data {
                    RecordData::A(_) => RecordType::A,
                    RecordData::AAAA(_) => RecordType::AAAA,
                    RecordData::CNAME(_) => RecordType::CNAME,
                    RecordData::NS(_) => RecordType::NS,
                    RecordData::MX(_) => RecordType::MX,
                    RecordData::TXT(_) => RecordType::TXT,
                    RecordData::SRV(_) => RecordType::SRV,
                    RecordData::TLSA(_) => RecordType::TLSA,
                    RecordData::CAA(_) => RecordType::CAA,
                },
                ttl.as_provider_value(),
                vec![data],
                self.zone.as_str(),
            )
            .await
    }

    /// Remove one matching value, leaving other values in the RRset intact.
    pub async fn remove(&self, name: &str, data: RecordData) -> Result<(), Error> {
        self.check_name(name)?;
        self.updater
            .remove_from_rrset(
                name,
                match &data {
                    RecordData::A(_) => RecordType::A,
                    RecordData::AAAA(_) => RecordType::AAAA,
                    RecordData::CNAME(_) => RecordType::CNAME,
                    RecordData::NS(_) => RecordType::NS,
                    RecordData::MX(_) => RecordType::MX,
                    RecordData::TXT(_) => RecordType::TXT,
                    RecordData::SRV(_) => RecordType::SRV,
                    RecordData::TLSA(_) => RecordType::TLSA,
                    RecordData::CAA(_) => RecordType::CAA,
                },
                vec![data],
                self.zone.as_str(),
            )
            .await
    }
}
