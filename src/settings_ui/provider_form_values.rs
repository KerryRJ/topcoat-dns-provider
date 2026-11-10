use super::ProviderFormProvider;

/// Non-secret values shown in the provider form.
#[derive(Clone, Debug)]
pub struct ProviderFormValues {
    pub enabled: bool,
    pub provider: ProviderFormProvider,
    pub zone: String,
    /// Cloudflare uses 1 for automatic TTL.
    pub ttl: String,
    /// Whether the selected provider already has a stored credential.
    pub credential_saved: bool,
}

impl Default for ProviderFormValues {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: ProviderFormProvider::Cloudflare,
            zone: String::new(),
            ttl: "3600".into(),
            credential_saved: false,
        }
    }
}
