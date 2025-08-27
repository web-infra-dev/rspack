use rspack_collections::IdentifierSet;
use rspack_error::Result;

#[async_trait::async_trait]
pub trait Backend: std::fmt::Debug + Send + Sync {
  async fn current_active_modules(&mut self) -> Result<IdentifierSet>;
}
