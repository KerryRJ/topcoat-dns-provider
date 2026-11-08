use crate::{
    ClouDnsAuthId, ClouDnsPassword, ClouDnsSubAuthId, DnsProvider, ZoneName, provider::ProviderKind,
};
use dns_update::{DnsUpdater, Error};
use std::time::Duration;

/// Choose the ClouDNS account or sub-account used for authentication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClouDnsAuth<'a> {
    AuthId(&'a ClouDnsAuthId),
    SubAuthId(&'a ClouDnsSubAuthId),
}

impl DnsProvider {
    /// Create a ClouDNS client using exactly one authentication ID and a password.
    pub fn cloudns(
        auth: ClouDnsAuth<'_>,
        password: &ClouDnsPassword,
        zone: ZoneName,
        timeout: Option<Duration>,
    ) -> Result<Self, Error> {
        let (auth_id, sub_auth_id) = match auth {
            ClouDnsAuth::AuthId(id) => (Some(id.as_str()), None),
            ClouDnsAuth::SubAuthId(id) => (None, Some(id.as_str())),
        };
        Ok(Self {
            updater: DnsUpdater::new_cloudns(auth_id, sub_auth_id, password.as_str(), timeout)?,
            zone,
            kind: ProviderKind::Cloudns,
        })
    }
}
