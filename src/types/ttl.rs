use std::str::FromStr;

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
