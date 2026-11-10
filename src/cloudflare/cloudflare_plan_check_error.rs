/// The zone subscription could not be verified. No credential or provider response is exposed.
#[derive(Debug)]
pub struct CloudflarePlanCheckError;

impl std::fmt::Display for CloudflarePlanCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Could not verify the Cloudflare zone subscription")
    }
}
impl std::error::Error for CloudflarePlanCheckError {}
