//! Asynchronous DNS record operations using a configured provider.
//!
//! A record's data determines its type; removal targets only that value.

mod cloudflare;
mod cloudns;
mod provider;
mod store;
mod types;

pub use cloudns::ClouDnsAuth;
pub use cloudflare::{CloudflarePlanCheckError, check_cloudflare_enterprise};
pub use dns_update::{
    CAARecord as Caa, DnsRecord as RecordData, DnsRecordType as RecordType, KeyValue,
    MXRecord as Mx, SRVRecord as Srv, TLSARecord as Tlsa, TlsaCertUsage, TlsaMatching,
    TlsaSelector,
};
pub use provider::DnsProvider;
pub use store::{
    ClouDnsAuthentication, DnsProviderSettings, DnsProviderStore, ProviderCredentials,
};
pub use types::{ClouDnsAuthId, ClouDnsPassword, ClouDnsSubAuthId, CloudflareToken, Ttl, ZoneName};

#[cfg(test)]
mod tests;

pub mod components;
pub mod fonts;
pub mod settings_ui;
