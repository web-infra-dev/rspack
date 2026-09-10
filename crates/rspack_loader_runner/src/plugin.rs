use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::InternedPathSet;
use rspack_sources::SourceMap;

use crate::{
  LoaderContext, LoaderRunnerContext,
  content::{Content, ResourceData},
};

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin: Send + Sync {
  type Context: LoaderRunnerContext;

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
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Option<(Content, Option<SourceMap<'static>>, InternedPathSet)>>;
}
