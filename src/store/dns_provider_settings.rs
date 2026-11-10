use super::{ClouDnsAuthentication, ProviderCredentials};

/// Complete settings, including credentials. Store these securely and never expose
/// this value directly in UI responses or logs.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DnsProviderSettings {
    /// Whether the host should use this configuration for DNS operations.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub zone: crate::ZoneName,
    /// TTL in seconds; Cloudflare uses 1 to indicate automatic TTL.
    pub ttl: crate::Ttl,
    pub provider: ProviderCredentials,
}

fn default_enabled() -> bool {
    true
}

impl DnsProviderSettings {
    /// Build a client only when enabled. Reload settings and rebuild the client
    /// after changes; previously constructed clients are not revoked.
    pub fn enabled_provider(
        &self,
        timeout: Option<std::time::Duration>,
    ) -> Result<Option<crate::DnsProvider>, dns_update::Error> {
        if !self.enabled {
            return Ok(None);
        }
        let provider = match &self.provider {
            ProviderCredentials::Cloudflare { token } => {
                crate::DnsProvider::cloudflare(token, self.zone.clone(), timeout)?
            }
            ProviderCredentials::Cloudns {
                authentication,
                password,
            } => {
                let auth = match authentication {
                    ClouDnsAuthentication::AuthId(id) => crate::ClouDnsAuth::AuthId(id),
                    ClouDnsAuthentication::SubAuthId(id) => crate::ClouDnsAuth::SubAuthId(id),
                };
                crate::DnsProvider::cloudns(auth, password, self.zone.clone(), timeout)?
            }
        };
        Ok(Some(provider))
    }
}
