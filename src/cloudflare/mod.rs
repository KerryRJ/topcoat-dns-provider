mod api_response;
mod cloudflare_plan_check_error;
mod rate_plan;
mod subscription;
mod zone;

pub use cloudflare_plan_check_error::CloudflarePlanCheckError;

use crate::{CloudflareToken, DnsProvider, ZoneName, provider::ProviderKind};
use api_response::ApiResponse;
use dns_update::{DnsUpdater, Error};
use std::{sync::OnceLock, time::Duration};
use subscription::{Subscription, classify_subscription};
use zone::Zone;

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
