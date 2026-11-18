use std::sync::Arc;

use rspack_error::{Diagnostic, Result};
use rspack_fs::ReadableFileSystem;
use rspack_sources::SourceMap;

use crate::{
  LoaderContext, LoaderDependencies,
  content::{Content, ResourceData},
};

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin: Send + Sync {
  type Context: Send;

  fn name(&self) -> &'static str {
    "unknown"
  }

  async fn before_all(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
    Ok(())
  }

  async fn start_yielding(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
    Ok(())
  }

  async fn run_normal_chain(&self, context: &mut LoaderContext<Self::Context>) -> Result<()> {
    context.run_normal_chain().await
  }

  async fn process_resource(
    &self,
    resource_data: &ResourceData,
    diagnostics: &mut Vec<Diagnostic>,
    dependencies: &mut LoaderDependencies,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Option<(Content, Option<SourceMap>)>>;
}
