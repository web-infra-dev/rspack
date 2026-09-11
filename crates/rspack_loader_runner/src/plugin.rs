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

  async fn before_all(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
    Ok(())
  }

  /// Transfer the same allocation to the foreign runner and back, including on
  /// loader errors. Native hooks and loaders only borrow the context.
  async fn start_yielding(
    &self,
    context: Box<LoaderContext<Self::Context>>,
  ) -> (Box<LoaderContext<Self::Context>>, Result<()>) {
    (context, Ok(()))
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

/// A foreign loader runner owns the context for the duration of its invocation.
#[async_trait::async_trait]
pub trait LoaderRunner: std::fmt::Debug + Send + Sync {
  type Context: LoaderRunnerContext;

  async fn run(
    &self,
    context: Box<LoaderContext<Self::Context>>,
  ) -> (Box<LoaderContext<Self::Context>>, Result<()>);
}
