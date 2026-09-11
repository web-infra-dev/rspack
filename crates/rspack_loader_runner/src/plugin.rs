use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::InternedPathSet;
use rspack_sources::SourceMap;

use crate::{
  Loader, LoaderContext, LoaderRunnerContext,
  content::{Content, ResourceData},
};

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin: Send + Sync {
  type Context: LoaderRunnerContext;

  fn name(&self) -> &'static str {
    "unknown"
  }

  /// Hooks may move the boxed context into JavaScript and must restore it before
  /// returning, including on error.
  async fn before_all(
    &self,
    _context: &mut Option<Box<LoaderContext<Self::Context>>>,
  ) -> Result<()> {
    Ok(())
  }

  /// A yielding plugin may take ownership of the boxed context, but must restore
  /// it before returning, including on error.
  async fn start_yielding(
    &self,
    _context: &mut Option<Box<LoaderContext<Self::Context>>>,
  ) -> Result<()> {
    Ok(())
  }

  async fn run_normal_loader(
    &self,
    context: &mut LoaderContext<Self::Context>,
    loader: Arc<dyn Loader<Self::Context>>,
  ) -> Result<()> {
    loader.run(context).await?;
    if !context.current_loader_state().finish_called() {
      context.finish_with_empty();
    }
    Ok(())
  }

  async fn process_resource(
    &self,
    resource_data: &ResourceData,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Option<(Content, Option<SourceMap<'static>>, InternedPathSet)>>;
}
