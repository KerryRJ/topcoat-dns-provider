use crate::ClouDnsAuthentication;

/// Provider-specific, non-secret values shown in the form.
#[derive(Clone, Debug)]
pub enum ProviderFormProvider {
    Cloudflare,
    Cloudns {
        authentication: ClouDnsAuthentication,
    },
}
