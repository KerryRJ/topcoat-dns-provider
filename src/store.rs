//! Application-provided persistence for DNS settings.
use std::future::Future;

/// Complete settings, including credentials. Store these securely and never expose
/// this value directly in UI responses or logs.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DnsProviderSettings {
    pub zone: crate::ZoneName,
    /// TTL in seconds; Cloudflare uses 1 to indicate automatic TTL.
    pub ttl: crate::Ttl,
    pub provider: ProviderCredentials,
}

/// Provider-specific credentials. These values must be encrypted at rest.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum ProviderCredentials {
    Cloudflare {
        token: crate::CloudflareToken,
    },
    Cloudns {
        authentication: ClouDnsAuthentication,
        password: crate::ClouDnsPassword,
    },
}

/// ClouDNS requires either an account ID or a sub-account ID.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum ClouDnsAuthentication {
    AuthId(crate::ClouDnsAuthId),
    SubAuthId(crate::ClouDnsSubAuthId),
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        convert::Infallible,
        sync::{Arc, Mutex},
        task::{Context, Poll, Wake, Waker},
    };

    #[derive(Default)]
    struct MemoryStore(Mutex<Option<DnsProviderSettings>>);
    impl DnsProviderStore for MemoryStore {
        type Error = Infallible;
        async fn load(&self) -> Result<Option<DnsProviderSettings>, Self::Error> {
            Ok(self.0.lock().unwrap().clone())
        }
        async fn save(&self, settings: &DnsProviderSettings) -> Result<(), Self::Error> {
            *self.0.lock().unwrap() = Some(settings.clone());
            Ok(())
        }
    }

    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn generic_store_round_trips_settings() {
        async fn exercise<S: DnsProviderStore<Error = Infallible>>(store: &S) {
            assert!(store.load().await.unwrap().is_none());
            let settings = DnsProviderSettings {
                zone: "example".parse().unwrap(),
                ttl: crate::Ttl::seconds(300).unwrap(),
                provider: ProviderCredentials::Cloudns {
                    authentication: ClouDnsAuthentication::SubAuthId("42".parse().unwrap()),
                    password: "secret".parse().unwrap(),
                },
            };
            store.save(&settings).await.unwrap();
            let loaded = store.load().await.unwrap().unwrap();
            assert_eq!(loaded.zone.as_str(), "example");
            assert_eq!(loaded.ttl, crate::Ttl::seconds(300).unwrap());
            assert!(
                matches!(loaded.provider, ProviderCredentials::Cloudns { authentication: ClouDnsAuthentication::SubAuthId(id), password } if id.as_str() == "42" && password.as_str() == "secret")
            );
        }
        let store = MemoryStore::default();
        let mut future = Box::pin(exercise(&store));
        let waker = Waker::from(Arc::new(NoopWake));
        assert!(matches!(
            future.as_mut().poll(&mut Context::from_waker(&waker)),
            Poll::Ready(())
        ));
    }
}
