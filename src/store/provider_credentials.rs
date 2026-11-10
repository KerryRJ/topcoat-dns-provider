use super::ClouDnsAuthentication;

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
