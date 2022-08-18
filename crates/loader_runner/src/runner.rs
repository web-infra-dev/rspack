use anyhow::Result;

use std::fmt::Debug;

use crate::Content;
// use crate::LoaderRunnerPlugin;

type Source = Content;

#[derive(Debug, Clone)]
pub struct ResourceData {
  pub resource: String,
  pub resource_path: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
}

#[derive(Debug)]
pub struct LoaderContext<'a> {
  pub source: Source,
  /// The resource part of the request, including query and fragment.
  ///
  /// E.g. /abc/resource.js?query=1#some-fragment
  pub resource: &'a str,
  /// The resource part of the request.
  ///
  /// E.g. /abc/resource.js
  pub resource_path: &'a str,
  /// The query of the request
  ///
  /// E.g. query=1
  pub resource_query: Option<&'a str>,
  /// The fragment of the request
  ///
  /// E.g. some-fragment
  pub resource_fragment: Option<&'a str>,
}

#[derive(Debug)]
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
pub trait Loader: Sync + Send + Debug {
  /// Loader name for debugging
  fn name(&self) -> &'static str {
    "unknown-loader"
  }

  /// Each loader should expose a `run` fn, which will be called by the loader runner.
  ///
  /// 1. If a loader returns an error, the loader runner will stop loading the resource.
  /// 2. If a loader returns a `None`, the result of the loader will be the same as the previous one.
  async fn run(&self, loader_context: &LoaderContext<'_>) -> Result<Option<LoaderResult>>;

  fn as_any(&self) -> &dyn std::any::Any;

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub type BoxedLoader = Box<dyn Loader>;
// type BoxedRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<LoaderResult>;

pub struct LoaderRunner {
  // plugins: Vec<BoxedRunnerPlugin>,
  resource_data: ResourceData,
}

impl LoaderRunner {
  pub fn new(
    resource_data: ResourceData,
    // plugins: Vec<BoxedRunnerPlugin>,
  ) -> Self {
    Self {
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
      resource_query: self.resource_data.resource_query.as_deref(),
      resource_fragment: self.resource_data.resource_fragment.as_deref(),
    };

    Ok(loader_context)
  }

  pub async fn run(&self, loaders: impl AsRef<[&dyn Loader]>) -> LoaderRunnerResult {
    let mut loader_context = self.get_loader_context().await?;

    tracing::debug!("Running loaders for resource: {}", loader_context.resource);

    for loader in loaders.as_ref().iter().rev() {
      tracing::debug!("Running loader: {}", loader.name());

      if let Some(loader_result) = loader.run(&loader_context).await? {
        loader_context.source = loader_result.content;
      }
    }

    Ok(loader_context.into())
  }
}
