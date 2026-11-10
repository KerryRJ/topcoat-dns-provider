#[derive(serde::Deserialize)]
pub(crate) struct PlanRequest {
    pub(crate) zone: String,
    pub(crate) credential: String,
}
