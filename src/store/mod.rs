//! Application-provided persistence for DNS settings.

mod clou_dns_authentication;
mod dns_provider_settings;
mod dns_provider_store;
mod provider_credentials;

pub use clou_dns_authentication::ClouDnsAuthentication;
pub use dns_provider_settings::DnsProviderSettings;
pub use dns_provider_store::DnsProviderStore;
pub use provider_credentials::ProviderCredentials;

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
    fn old_settings_default_to_enabled() {
        let settings: DnsProviderSettings = serde_json::from_str(
            r#"{"zone":"example.com","ttl":300,"provider":{"provider":"cloudflare","token":"secret"}}"#,
        ).unwrap();
        assert!(settings.enabled);
    }

    #[test]
    fn generic_store_round_trips_settings() {
        async fn exercise<S: DnsProviderStore<Error = Infallible>>(store: &S) {
            assert!(store.load().await.unwrap().is_none());
            let settings = DnsProviderSettings {
                enabled: false,
                zone: "example".parse().unwrap(),
                ttl: crate::Ttl::seconds(300).unwrap(),
                provider: ProviderCredentials::Cloudns {
                    authentication: ClouDnsAuthentication::SubAuthId("42".parse().unwrap()),
                    password: "secret".parse().unwrap(),
                },
            };
            store.save(&settings).await.unwrap();
            let loaded = store.load().await.unwrap().unwrap();
            assert!(!loaded.enabled);
            assert!(loaded.enabled_provider(None).unwrap().is_none());
            let mut enabled = loaded.clone();
            enabled.enabled = true;
            assert!(enabled.enabled_provider(None).unwrap().is_some());
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
