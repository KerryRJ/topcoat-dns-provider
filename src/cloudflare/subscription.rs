use super::{CloudflarePlanCheckError, rate_plan::RatePlan};

#[derive(serde::Deserialize)]
pub(super) struct Subscription {
    rate_plan: Option<RatePlan>,
    state: Option<String>,
}

pub(super) fn classify_subscription(
    subscription: Subscription,
) -> Result<bool, CloudflarePlanCheckError> {
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
mod tests {
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
