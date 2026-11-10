mod clou_dns_password;
mod cloudflare_token;
mod ttl;
mod zone_name;

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
        impl std::str::FromStr for $name {
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

mod clou_dns_auth_id;
mod clou_dns_sub_auth_id;

pub use clou_dns_auth_id::ClouDnsAuthId;
pub use clou_dns_password::ClouDnsPassword;
pub use clou_dns_sub_auth_id::ClouDnsSubAuthId;
pub use cloudflare_token::CloudflareToken;
pub use ttl::Ttl;
pub use zone_name::ZoneName;
