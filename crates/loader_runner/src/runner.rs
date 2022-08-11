use anyhow::Result;

use crate::Content;
// use crate::LoaderRunnerPlugin;

type Source = Content;

pub struct ResourceData {
  pub resource: String,
  pub resource_path: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
}

pub struct LoaderContext<'a> {
  pub source: Source,
  pub resource: &'a str,
  pub resource_path: &'a str,
  pub resource_query: Option<&'a str>,
  pub resource_fragment: Option<&'a str>,
}

pub struct LoaderResult {
  pub content: Content,
}

impl From<LoaderContext<'_>> for LoaderResult {
  fn from(loader_context: LoaderContext<'_>) -> Self {
    Self {
      content: loader_context.source,
    }
  }
}

#[async_trait::async_trait]
pub trait Loader: Sync + Send {
  /// Loader name for debugging
  fn name(&self) -> &'static str {
    "unknown-loader"
  }

  /// Each loader should expose a `run` fn, which will be called by the loader runner.
  ///
  /// 1. If a loader returns an error, the loader runner will stop loading the resource.
  /// 2. If a loader returns an `None`, the result of the loader will be the same as the previous one.
  async fn run<'a>(&self, loader_context: &LoaderContext<'a>) -> Result<Option<LoaderResult>>;
}

type BoxedLoader = Box<dyn Loader>;
// type BoxedRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<LoaderResult>;

pub struct LoaderRunner {
  loaders: Vec<BoxedLoader>,
  // plugins: Vec<BoxedRunnerPlugin>,
  resource_data: ResourceData,
}

impl LoaderRunner {
  pub fn new(
    resource_data: ResourceData,
    loaders: Vec<BoxedLoader>,
    // plugins: Vec<BoxedRunnerPlugin>,
  ) -> Self {
    Self {
      loaders,
      // plugins,
      resource_data,
    }
  }

  async fn process_resource(&self) -> Result<Content> {
    let result = tokio::fs::read(&self.resource_data.resource_path).await?;
    Ok(Content::from(result))
  }

  async fn get_loader_context(&self) -> Result<LoaderContext> {
    let content = self.process_resource().await?;

    let loader_context = LoaderContext {
      source: content,
      resource: &self.resource_data.resource,
      resource_path: &self.resource_data.resource_path,
      resource_query: self
        .resource_data
        .resource_query
        .as_ref()
        .map(|s| s.as_str()),
      resource_fragment: self
        .resource_data
        .resource_fragment
        .as_ref()
        .map(|s| s.as_str()),
    };

    Ok(loader_context)
  }

  pub async fn run(&mut self) -> LoaderRunnerResult {
    let mut loader_context = self.get_loader_context().await?;

    tracing::debug!("Running loaders for resource: {}", loader_context.resource);

    for loader in self.loaders.iter().rev() {
      tracing::debug!("Running loader: {}", loader.name());

      if let Some(loader_result) = loader.run(&loader_context).await? {
        loader_context.source = loader_result.content;
      }
    }

    Ok(loader_context.into())
  }
}
