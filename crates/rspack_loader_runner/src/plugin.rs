use rspack_error::Result;

use crate::{content::Content, runner::ResourceData, LoaderContext};

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin: Send + Sync {
  type Context;

  fn name(&self) -> &'static str {
    "unknown"
  }

  fn loader_context(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()>;

  async fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>>;
}
