use rspack_error::Result;

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
pub struct LoaderContext<'a, T, U> {
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

  pub compiler_context: T,
  pub compilation_context: U,
}

#[derive(Debug)]
pub struct LoaderResult {
  pub content: Content,
}

impl<T, U> From<LoaderContext<'_, T, U>> for LoaderResult {
  fn from(loader_context: LoaderContext<'_, T, U>) -> Self {
    Self {
      content: loader_context.source,
    }
  }
}

#[async_trait::async_trait]
pub trait Loader<T, U>: Sync + Send + Debug {
  /// Loader name for debugging
  fn name(&self) -> &'static str {
    "unknown-loader"
  }

  /// Each loader should expose a `run` fn, which will be called by the loader runner.
  ///
  /// 1. If a loader returns an error, the loader runner will stop loading the resource.
  /// 2. If a loader returns a `None`, the result of the loader will be the same as the previous one.
  async fn run(&self, loader_context: &LoaderContext<'_, T, U>) -> Result<Option<LoaderResult>>;

  fn as_any(&self) -> &dyn std::any::Any;

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub type BoxedLoader<T, U> = Box<dyn Loader<T, U>>;
// type BoxedRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<LoaderResult>;

pub struct LoaderRunner<'a, T, U> {
  // plugins: Vec<BoxedRunnerPlugin>,
  resource_data: ResourceData,
  compiler_context: &'a T,
  compilation_context: &'a U,
}

impl<T, U> LoaderRunner<'_, T, U> {
  pub fn new(
    resource_data: ResourceData,
    compiler_context: &T,
    compilation_context: &U, // plugins: Vec<BoxedRunnerPlugin>,
  ) -> Self {
    Self {
      // plugins,
      resource_data,
      compiler_context,
      compilation_context,
    }
  }

  async fn process_resource(&self) -> Result<Content> {
    let result = tokio::fs::read(&self.resource_data.resource_path).await?;
    Ok(Content::from(result))
  }

  async fn get_loader_context(&self) -> Result<LoaderContext<'_, T, U>> {
    let content = self.process_resource().await?;

    let loader_context = LoaderContext {
      source: content,
      resource: &self.resource_data.resource,
      resource_path: &self.resource_data.resource_path,
      resource_query: self.resource_data.resource_query.as_deref(),
      resource_fragment: self.resource_data.resource_fragment.as_deref(),
      compiler_context: &self.compiler_context,
      compilation_context: &self.compilation_context,
    };

    Ok(loader_context)
  }

  pub async fn run(&self, loaders: impl AsRef<[&dyn Loader<T, U>]>) -> LoaderRunnerResult {
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
