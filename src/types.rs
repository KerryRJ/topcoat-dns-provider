use std::{fmt, str::FromStr};

/// A normalized ASCII DNS zone name. Unicode names must be converted to IDNA
/// ASCII form by the caller. A single trailing root dot is accepted and removed.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct ZoneName(String);

impl ZoneName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for ZoneName {
    type Err = &'static str;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let name = input.trim().strip_suffix('.').unwrap_or(input.trim());
        if name.is_empty()
            || name.len() > 253
            || !name.is_ascii()
            || name.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
                    || !label
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return Err(
                "Enter a valid ASCII DNS zone name (use IDNA ASCII for international names)",
            );
        }
        Ok(Self(name.to_ascii_lowercase()))
    }
}
impl fmt::Display for ZoneName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for ZoneName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

/// A Cloudflare API token. Debug output is redacted; store it securely.
#[derive(Clone, Eq, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct CloudflareToken(String);

impl CloudflareToken {
    /// Exposes the token for passing to a provider client or secure storage.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for CloudflareToken {
    type Err = &'static str;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.trim().is_empty() {
            return Err("Cloudflare token cannot be empty");
        }
        Ok(Self(input.to_owned()))
    }
}
impl fmt::Debug for CloudflareToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CloudflareToken([REDACTED])")
    }
}
impl<'de> serde::Deserialize<'de> for CloudflareToken {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

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

macro_rules! cloudns_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl FromStr for $name {
            type Err = &'static str;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                if input.trim().is_empty() {
                    return Err("ClouDNS authentication ID cannot be empty");
                }
                Ok(Self(input.to_owned()))
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <String as serde::Deserialize>::deserialize(deserializer)?;
                value.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

cloudns_id!(
    ClouDnsAuthId,
    "A non-empty ClouDNS account authentication ID."
);
cloudns_id!(
    ClouDnsSubAuthId,
    "A non-empty ClouDNS sub-account authentication ID."
);

/// A DNS TTL in seconds, or Cloudflare's automatic TTL. Values are encoded as
/// numbers for compatibility with existing stored settings; 1 means automatic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ttl(TtlValue);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TtlValue {
    Auto,
    Seconds(std::num::NonZeroU32),
}
impl Ttl {
    /// Cloudflare automatic TTL.
    pub const AUTO: Self = Self(TtlValue::Auto);
    pub fn seconds(seconds: u32) -> Result<Self, &'static str> {
        match seconds {
            0 | 1 => Err("TTL seconds must be at least 2; use Auto for Cloudflare automatic TTL"),
            n => Ok(Self(TtlValue::Seconds(
                std::num::NonZeroU32::new(n).unwrap(),
            ))),
        }
    }
    pub fn as_provider_value(self) -> u32 {
        match self.0 {
            TtlValue::Auto => 1,
            TtlValue::Seconds(n) => n.get(),
        }
    }
    /// Allows the Enterprise minimum; Cloudflare may reject values below 60 on other plans.
    pub fn validate_cloudflare(self) -> Result<(), &'static str> {
        match self.0 {
            TtlValue::Auto => Ok(()),
            TtlValue::Seconds(n) if (30..=86400).contains(&n.get()) => Ok(()),
            _ => Err("Cloudflare TTL must be automatic or between 30 and 86400 seconds"),
        }
    }
    pub fn validate_cloudns(self) -> Result<(), &'static str> {
        const VALUES: [u32; 13] = [
            60, 300, 900, 1800, 3600, 21600, 43200, 86400, 172800, 259200, 604800, 1209600, 2592000,
        ];
        match self.0 {
            TtlValue::Seconds(n) if VALUES.contains(&n.get()) => Ok(()),
            _ => Err("Choose a TTL supported by ClouDNS"),
        }
    }
}
impl FromStr for Ttl {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().parse::<u32>() {
            Ok(1) => Ok(Self::AUTO),
            Ok(n) => Self::seconds(n),
            Err(_) => Err("Enter a valid TTL in seconds"),
        }
    }
}
impl serde::Serialize for Ttl {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.as_provider_value())
    }
}
impl<'de> serde::Deserialize<'de> for Ttl {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let n = <u32 as serde::Deserialize>::deserialize(deserializer)?;
        if n == 1 {
            Ok(Self::AUTO)
        } else {
            Self::seconds(n).map_err(serde::de::Error::custom)
        }
    }
}
