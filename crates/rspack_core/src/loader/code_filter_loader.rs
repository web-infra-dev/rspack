use std::{fmt, sync::Arc};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_error::Result;
use rspack_loader_runner::{CodeFilter, Loader, LoaderContext, LoaderExecutionKind};

use crate::{BoxLoader, RunnerContext};

#[cacheable]
pub struct CodeFilteredLoader {
  target: BoxLoader,
  filter: Arc<CodeFilter>,
}

impl CodeFilteredLoader {
  pub fn new(target: BoxLoader, filter: Arc<CodeFilter>) -> Self {
    Self { target, filter }
  }
}

impl fmt::Debug for CodeFilteredLoader {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CodeFilteredLoader")
      .field("target", &self.target.identifier())
      .field("filter", &self.filter)
      .finish()
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for CodeFilteredLoader {
  fn identifier(&self) -> Identifier {
    self.target.identifier()
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    self.target.run(loader_context).await
  }

  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    self.target.pitch(loader_context).await
  }

  fn r#type(&self) -> Option<&str> {
    self.target.r#type()
  }

  fn cache_version(&self) -> Option<&str> {
    self.target.cache_version()
  }

  fn execution_kind(&self) -> LoaderExecutionKind {
    self.target.execution_kind()
  }

  fn code_filter(&self) -> Option<&CodeFilter> {
    Some(&self.filter)
  }
}
