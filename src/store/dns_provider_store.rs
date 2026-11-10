use super::DnsProviderSettings;
use std::future::Future;

/// Implemented by a host application for its chosen database and security model.
///
/// A store instance identifies its own application or tenant scope. `load`
/// returns `None` when no settings exist. `save` replaces the complete settings,
/// including credentials; retaining an existing secret is the caller's responsibility.
/// The host must authorize access and protect secrets at rest and in transit.
/// Futures are `Send` so they can be used in multithreaded async servers.
/// This trait is intended for generic use and is not object-safe.
pub trait DnsProviderStore: Send + Sync {
    type Error;

    fn load(&self)
    -> impl Future<Output = Result<Option<DnsProviderSettings>, Self::Error>> + Send;
    fn save(
        &self,
        settings: &DnsProviderSettings,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
