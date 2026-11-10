use std::{fmt, str::FromStr};

/// A ClouDNS API password. Debug output is redacted; store it securely.
#[derive(Clone, Eq, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct ClouDnsPassword(String);

impl ClouDnsPassword {
    /// Exposes the password for passing to a provider client or secure storage.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for ClouDnsPassword {
    type Err = &'static str;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.trim().is_empty() {
            return Err("ClouDNS password cannot be empty");
        }
        Ok(Self(input.to_owned()))
    }
}
impl fmt::Debug for ClouDnsPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClouDnsPassword([REDACTED])")
    }
}
impl<'de> serde::Deserialize<'de> for ClouDnsPassword {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}
