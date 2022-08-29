use rspack_error::Result;

use std::fmt::Debug;

use crate::{Content, LoaderRunnerPlugin};

type Source = Content;

#[derive(Debug, Clone)]
pub struct ResourceData {
  pub resource: String,
  pub resource_path: String,
  pub resource_query: Option<String>,
  pub resource_fragment: Option<String>,
}

#[derive(Debug)]
pub struct LoaderContext<'a, 'context, T, U> {
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

  pub compiler_context: &'context T,

  pub compilation_context: &'context U,

  pub extra_data: Option<String>,
}

#[derive(Debug)]
pub struct LoaderResult {
  pub content: Content,
  pub extra_data: Option<String>,
}

impl<T, U> From<LoaderContext<'_, '_, T, U>> for LoaderResult {
  fn from(loader_context: LoaderContext<'_, '_, T, U>) -> Self {
    Self {
      content: loader_context.source,
      extra_data: loader_context.extra_data,
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
  async fn run(&self, loader_context: &LoaderContext<'_, '_, T, U>)
    -> Result<Option<LoaderResult>>;

  fn as_any(&self) -> &dyn std::any::Any;

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub type BoxedLoader<T, U> = Box<dyn Loader<T, U>>;
pub type BoxedRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<LoaderResult>;

pub struct LoaderRunner {
  plugins: Vec<BoxedRunnerPlugin>,
  resource_data: ResourceData,
}

#[derive(Debug)]
pub struct LoaderRunnerAdditionalContext<'context, T, U> {
  pub compiler: &'context T,
  pub compilation: &'context U,
}

impl LoaderRunner {
  pub fn new(resource_data: ResourceData, plugins: Vec<BoxedRunnerPlugin>) -> Self {
    Self {
      plugins,
      resource_data,
    }
  }

  /// Process resource
  ///
  /// Plugins are loaded in order, if a plugin returns `Some(Content)`, then the returning content will be used as the result.
  /// If plugins returned nothing, the runner will read via the `resource_path`.
  async fn process_resource(&self) -> Result<Content> {
    for plugin in &self.plugins {
      if let Some(processed_resource) = plugin.process_resource(&self.resource_data).await? {
        return Ok(processed_resource);
      }
    }

    let result = tokio::fs::read(&self.resource_data.resource_path).await?;
    Ok(Content::from(result))
  }

  async fn get_loader_context<'context, T, U>(
    &self,
    context: &'context LoaderRunnerAdditionalContext<'_, T, U>,
  ) -> Result<LoaderContext<'_, 'context, T, U>> {
    let content = self.process_resource().await?;

    let loader_context = LoaderContext {
      source: content,
      resource: &self.resource_data.resource,
      resource_path: &self.resource_data.resource_path,
      resource_query: self.resource_data.resource_query.as_deref(),
      resource_fragment: self.resource_data.resource_fragment.as_deref(),
      compiler_context: context.compiler,
      compilation_context: context.compilation,
      extra_data: None,
    };

    Ok(loader_context)
  }

  pub async fn run<'loader, 'context: 'loader, T, U>(
    &self,
    loaders: impl AsRef<[&'loader dyn Loader<T, U>]>,
    context: &'context LoaderRunnerAdditionalContext<'_, T, U>,
  ) -> LoaderRunnerResult {
    let mut loader_context = self.get_loader_context(context).await?;

    tracing::debug!("Running loaders for resource: {}", loader_context.resource);

    for loader in loaders.as_ref().iter().rev() {
      tracing::debug!("Running loader: {}", loader.name());

      if let Some(loader_result) = loader.run(&loader_context).await? {
        loader_context.source = loader_result.content;
        loader_context.extra_data = loader_result.extra_data;
      }
    }

    Ok(loader_context.into())
  }
}
