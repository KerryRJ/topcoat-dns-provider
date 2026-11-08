use crate::{CloudflareToken, DnsProvider, ZoneName, provider::ProviderKind};
use dns_update::{DnsUpdater, Error};
use std::{sync::OnceLock, time::Duration};

/// The zone subscription could not be verified. No credential or provider response is exposed.
#[derive(Debug)]
pub struct CloudflarePlanCheckError;

impl std::fmt::Display for CloudflarePlanCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Could not verify the Cloudflare zone subscription")
    }
}
impl std::error::Error for CloudflarePlanCheckError {}

#[derive(serde::Deserialize)]
struct ApiResponse<T> {
    success: bool,
    result: T,
}
#[derive(serde::Deserialize)]
struct Zone {
    id: String,
    name: String,
}
#[derive(serde::Deserialize)]
struct Subscription {
    rate_plan: Option<RatePlan>,
    state: Option<String>,
}
#[derive(serde::Deserialize)]
struct RatePlan {
    id: Option<String>,
}

/// Check whether the exact zone has an active Enterprise subscription. This performs
/// read-only Cloudflare API requests and requires zone lookup and Billing Read access.
/// An unknown subscription state or plan is an error, not a negative confirmation.
pub async fn check_cloudflare_enterprise(
    token: &CloudflareToken,
    zone: &ZoneName,
) -> Result<bool, CloudflarePlanCheckError> {
    static CLIENT: OnceLock<Result<reqwest::Client, reqwest::Error>> = OnceLock::new();
    let client = CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
        })
        .as_ref()
        .map_err(|_| CloudflarePlanCheckError)?;
    let zones: ApiResponse<Vec<Zone>> = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .bearer_auth(token.as_str())
        .query(&[("name", zone.as_str())])
        .send()
        .await
        .map_err(|_| CloudflarePlanCheckError)?
        .error_for_status()
        .map_err(|_| CloudflarePlanCheckError)?
        .json()
        .await
        .map_err(|_| CloudflarePlanCheckError)?;
    if !zones.success {
        return Err(CloudflarePlanCheckError);
    }
    let mut matches = zones
        .result
        .iter()
        .filter(|z| z.name.eq_ignore_ascii_case(zone.as_str()));
    let zone_id = &matches.next().ok_or(CloudflarePlanCheckError)?.id;
    if matches.next().is_some()
        || zone_id.len() != 32
        || !zone_id.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(CloudflarePlanCheckError);
    }
    let url = format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/subscription");
    let subscription: ApiResponse<Subscription> = client
        .get(url)
        .bearer_auth(token.as_str())
        .send()
        .await
        .map_err(|_| CloudflarePlanCheckError)?
        .error_for_status()
        .map_err(|_| CloudflarePlanCheckError)?
        .json()
        .await
        .map_err(|_| CloudflarePlanCheckError)?;
    if !subscription.success {
        return Err(CloudflarePlanCheckError);
    }
    classify_subscription(subscription.result)
}

fn classify_subscription(subscription: Subscription) -> Result<bool, CloudflarePlanCheckError> {
    let state = subscription
        .state
        .as_deref()
        .ok_or(CloudflarePlanCheckError)?;
    if !matches!(state, "Trial" | "Provisioned" | "Paid") {
        return Err(CloudflarePlanCheckError);
    }
    let id = subscription
        .rate_plan
        .and_then(|p| p.id)
        .ok_or(CloudflarePlanCheckError)?;
    match id.as_str() {
        "enterprise" | "partners_enterprise" => Ok(true),
        "free" | "lite" | "pro" | "pro_plus" | "business" | "partners_free" | "partners_pro"
        | "partners_business" => Ok(false),
        _ => Err(CloudflarePlanCheckError),
    }
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    #[test]
    fn only_recognized_active_enterprise_plans_are_accepted() {
        let plan = |id: Option<&str>, state: Option<&str>| Subscription {
            rate_plan: id.map(|id| RatePlan {
                id: Some(id.into()),
            }),
            state: state.map(str::to_owned),
        };
        assert!(classify_subscription(plan(Some("enterprise"), Some("Paid"))).unwrap());
        assert!(
            classify_subscription(plan(Some("partners_enterprise"), Some("Provisioned"))).unwrap()
        );
        assert!(!classify_subscription(plan(Some("business"), Some("Paid"))).unwrap());
        assert!(classify_subscription(plan(Some("enterprise"), Some("Expired"))).is_err());
        assert!(classify_subscription(plan(Some("unknown"), Some("Paid"))).is_err());
        assert!(classify_subscription(plan(None, Some("Paid"))).is_err());
    }
}

impl DnsProvider {
    /// Create a Cloudflare client. The token must have access to the specified zone.
    pub fn cloudflare(
        token: &CloudflareToken,
        zone: ZoneName,
        timeout: Option<Duration>,
    ) -> Result<Self, Error> {
        Ok(Self {
            updater: DnsUpdater::new_cloudflare(token.as_str(), timeout)?,
            zone,
            kind: ProviderKind::Cloudflare,
        })
    }
}
