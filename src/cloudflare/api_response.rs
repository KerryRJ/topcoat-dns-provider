#[derive(serde::Deserialize)]
pub(super) struct ApiResponse<T> {
    pub(super) success: bool,
    pub(super) result: T,
}
