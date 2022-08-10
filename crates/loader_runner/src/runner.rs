use anyhow::Result;

use crate::plugin::LoaderRunnerPlugin;

pub struct LoaderContext {}

pub struct LoaderResult {}

#[async_trait::async_trait]
trait Loader: Sync + Send {
  fn name(&self) -> &'static str {
    "unknown-loader"
  }

  async fn run(&self) -> Result<Option<LoaderResult>>;
}

type BoxedLoader = Box<dyn Loader>;
type BoxedRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<LoaderResult>;

pub struct LoaderRunner {
  loaders: Vec<BoxedLoader>,
  plugins: Vec<BoxedRunnerPlugin>,
}

impl LoaderRunner {
  fn new(resource_data: (), loaders: Vec<BoxedLoader>, plugins: Vec<BoxedRunnerPlugin>) -> Self {
    Self { loaders, plugins }
  }

  fn run(&self) -> LoaderRunnerResult {
    todo!()
  }
}
