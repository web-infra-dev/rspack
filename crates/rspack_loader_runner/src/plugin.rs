use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::InternedPathSet;
use rspack_sources::SourceMap;

use crate::{
  LoaderContext, LoaderYieldResult,
  content::{AdditionalData, Content, ResourceData},
};

pub struct LoaderYieldInput {
  pub content: Option<Content>,
  pub additional_data: Option<AdditionalData>,
}

impl LoaderYieldInput {
  pub fn restore<Context: Send>(self, context: &mut LoaderContext<Context>) {
    context.content = self.content;
    context.additional_data = self.additional_data;
  }
}

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin: Send + Sync {
  type Context: Send + Sync;

  fn name(&self) -> &'static str {
    "unknown"
  }

  async fn before_all(&self, _context: &mut LoaderContext<Self::Context>) -> Result<()> {
    Ok(())
  }

  async fn start_yielding(
    &self,
    _context: &Arc<LoaderContext<Self::Context>>,
    input: LoaderYieldInput,
  ) -> Result<LoaderYieldResult> {
    Ok(LoaderYieldResult::Unhandled(input))
  }

  async fn run_normal_chain(&self, context: &mut Arc<LoaderContext<Self::Context>>) -> Result<()> {
    LoaderContext::run_normal_chain(context).await
  }

  async fn process_resource(
    &self,
    resource_data: &ResourceData,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Option<(Content, Option<SourceMap<'static>>, InternedPathSet)>>;
}
