use std::sync::{Arc, OnceLock};

use derive_more::Debug;
use rspack_cacheable::{cacheable, with::Skip};
use rspack_error::Error;
use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::{
  LoaderResult, LoaderRunnerData, LoaderRunnerPlugin, ResourceData, run_loaders_with_data,
};

use crate::{BoxLoader, RunnerContext};

/// Static loader metadata and its preassembled execution plan for one module.
#[cacheable]
#[derive(Debug)]
pub struct Loaders {
  #[debug(skip)]
  loaders: Vec<BoxLoader>,
  #[cacheable(with=Skip)]
  loader_data: OnceLock<Arc<LoaderRunnerData<RunnerContext>>>,
}

impl Loaders {
  pub(crate) fn new(loaders: Vec<BoxLoader>) -> Self {
    Self {
      loaders,
      loader_data: OnceLock::new(),
    }
  }

  pub fn loaders(&self) -> &[BoxLoader] {
    &self.loaders
  }

  fn loader_data(&self) -> Arc<LoaderRunnerData<RunnerContext>> {
    self
      .loader_data
      .get_or_init(|| {
        Arc::new(LoaderRunnerData::new(
          self.loaders.iter().cloned().collect(),
        ))
      })
      .clone()
  }

  pub async fn run_loaders(
    &self,
    resource_data: Arc<ResourceData>,
    plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = RunnerContext>>>,
    context: RunnerContext,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> (LoaderResult<RunnerContext>, Option<Error>) {
    run_loaders_with_data(self.loader_data(), resource_data, plugin, context, fs).await
  }
}
