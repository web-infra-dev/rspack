use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::InternedPathSet;
use rspack_sources::SourceMap;

use crate::{
  LoaderContext,
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

  /// Transfers ownership to the foreign runner and returns it, including on errors.
  async fn start_yielding(
    &self,
    context: Box<LoaderContext<Self::Context>>,
  ) -> (Box<LoaderContext<Self::Context>>, Result<()>) {
    (context, Ok(()))
  }

  async fn run_normal_chain(
    &self,
    context: Box<LoaderContext<Self::Context>>,
  ) -> (Box<LoaderContext<Self::Context>>, Result<()>) {
    context.run_normal_chain().await
  }

  async fn process_resource(
    &self,
    resource_data: &ResourceData,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Option<(Content, Option<SourceMap<'static>>, InternedPathSet)>>;
}
