use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{Result, TWithDiagnosticArray};
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{Content, LoaderRunnerPlugin};

type Source = Content;

#[derive(Debug, Clone)]
pub struct ResourceData {
  pub resource: String,
  pub resource_path: PathBuf,
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
  pub resource_path: &'a Path,
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

  pub source_map: Option<SourceMap>,

  pub additional_data: Option<String>,

  pub cacheable: bool,

  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
}

#[derive(Debug)]
pub struct LoaderResult {
  pub cacheable: bool,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub content: Content,
  pub source_map: Option<SourceMap>,
  pub additional_data: Option<String>,
}

impl LoaderResult {
  pub fn new(content: Content, source_map: Option<SourceMap>) -> Self {
    Self {
      cacheable: true,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
      content,
      source_map,
      additional_data: Default::default(),
    }
  }

  pub fn cacheable(mut self, v: bool) -> Self {
    self.cacheable = v;
    self
  }

  pub fn file_dependency(mut self, v: PathBuf) -> Self {
    self.file_dependencies.insert(v);
    self
  }

  pub fn context_dependency(mut self, v: PathBuf) -> Self {
    self.context_dependencies.insert(v);
    self
  }

  pub fn missing_dependency(mut self, v: PathBuf) -> Self {
    self.missing_dependencies.insert(v);
    self
  }

  pub fn build_dependency(mut self, v: PathBuf) -> Self {
    self.build_dependencies.insert(v);
    self
  }

  pub fn additional_data(mut self, v: String) -> Self {
    self.additional_data = Some(v);
    self
  }
}

impl<T, U> From<LoaderContext<'_, '_, T, U>> for LoaderResult {
  fn from(loader_context: LoaderContext<'_, '_, T, U>) -> Self {
    Self {
      cacheable: loader_context.cacheable,
      file_dependencies: loader_context.file_dependencies,
      context_dependencies: loader_context.context_dependencies,
      missing_dependencies: loader_context.missing_dependencies,
      build_dependencies: loader_context.build_dependencies,
      content: loader_context.source,
      source_map: loader_context.source_map,
      additional_data: loader_context.additional_data,
    }
  }
}

#[async_trait::async_trait]
pub trait Loader<T, U>: Sync + Send + Debug {
  /// Loader name for module identifier
  fn name(&self) -> &str;

  /// Each loader should expose a `run` fn, which will be called by the loader runner.
  ///
  /// 1. If a loader returns an error, the loader runner will stop loading the resource.
  /// 2. If a loader returns a `None`, the result of the loader will be the same as the previous one.
  async fn run(
    &self,
    loader_context: &LoaderContext<'_, '_, T, U>,
  ) -> Result<Option<TWithDiagnosticArray<LoaderResult>>>;

  fn as_any(&self) -> &dyn std::any::Any;

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub type BoxedLoader<T, U> = Arc<dyn Loader<T, U>>;
pub type BoxedRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<TWithDiagnosticArray<LoaderResult>>;

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

    // TODO: FileUriPlugin
    let mut file_dependencies: HashSet<PathBuf> = Default::default();
    file_dependencies.insert(self.resource_data.resource_path.clone());

    let loader_context = LoaderContext {
      cacheable: true,
      file_dependencies,
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
      source: content,
      resource: &self.resource_data.resource,
      resource_path: &self.resource_data.resource_path,
      resource_query: self.resource_data.resource_query.as_deref(),
      resource_fragment: self.resource_data.resource_fragment.as_deref(),
      compiler_context: context.compiler,
      compilation_context: context.compilation,
      source_map: None,
      additional_data: None,
    };

    Ok(loader_context)
  }

  pub async fn run<'loader, 'context: 'loader, T, U>(
    &self,
    loaders: impl AsRef<[&'loader dyn Loader<T, U>]>,
    context: &'context LoaderRunnerAdditionalContext<'_, T, U>,
  ) -> LoaderRunnerResult {
    let mut loader_context = self.get_loader_context(context).await?;
    let mut diagnostics = Vec::new();

    tracing::trace!("Running loaders for resource: {}", loader_context.resource);

    for loader in loaders.as_ref().iter().rev() {
      tracing::trace!("Running loader: {}", loader.name());

      if let Some(loader_result) = loader.run(&loader_context).await? {
        let (loader_result, ds) = loader_result.split_into_parts();
        loader_context.cacheable = loader_result.cacheable;
        loader_context.source = loader_result.content;
        loader_context.source_map = loader_result.source_map;
        loader_context.additional_data = loader_result.additional_data;
        loader_context.file_dependencies = loader_result.file_dependencies;
        loader_context.context_dependencies = loader_result.context_dependencies;
        loader_context.missing_dependencies = loader_result.missing_dependencies;
        loader_context.build_dependencies = loader_result.build_dependencies;
        diagnostics.extend(ds);
      }
    }

    Ok(TWithDiagnosticArray::new(
      loader_context.into(),
      diagnostics,
    ))
  }
}
