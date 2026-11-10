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
