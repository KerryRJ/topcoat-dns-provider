/// ClouDNS requires either an account ID or a sub-account ID.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum ClouDnsAuthentication {
    AuthId(crate::ClouDnsAuthId),
    SubAuthId(crate::ClouDnsSubAuthId),
}
