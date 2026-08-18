use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  LoaderCacheAction, LoaderCacheState, LoaderContext,
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

  async fn should_yield(&self, _context: &LoaderContext<Self::Context>) -> Result<bool> {
    Ok(false)
  }

  async fn start_yielding(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
    Ok(())
  }

  async fn before_normal_loader(
    &self,
    _context: &mut LoaderContext<Self::Context>,
  ) -> Result<LoaderCacheAction> {
    Ok(LoaderCacheAction::Disabled)
  }

  async fn after_normal_loader(
    &self,
    _context: &mut LoaderContext<Self::Context>,
    _state: LoaderCacheState,
  ) -> Result<()> {
    Ok(())
  }

  async fn process_resource(
    &self,
    resource_data: &ResourceData,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<
    Option<(
      Content,
      Option<SourceMap<'static>>,
      HashSet<std::path::PathBuf>,
    )>,
  >;
}
