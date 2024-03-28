use rspack_core::ModuleIdentifier;
use rspack_error::Result;

pub struct ModuleInfo {
  pub active: bool,
  pub data: String,
  pub client: String,
}

#[async_trait::async_trait]
pub trait Backend: std::fmt::Debug + Send + Sync {
  async fn module(&mut self, original_module: ModuleIdentifier, path: String)
    -> Result<ModuleInfo>;
}
