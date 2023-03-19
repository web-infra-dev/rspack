use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{Content, LoaderRunnerPlugin};

#[derive(Debug, Clone)]
pub struct ResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub resource_path: PathBuf,
  /// Resource query with `?` prefix
  pub resource_query: Option<String>,
  /// Resource fragment with `#` prefix
  pub resource_fragment: Option<String>,
  pub resource_description: Option<Arc<nodejs_resolver::DescriptionData>>,
}

#[derive(Debug)]
pub struct LoaderContext<'a, 'context, T, U> {
  pub content: Content,
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

  pub diagnostic: Vec<Diagnostic>,
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

impl<T, U> From<LoaderContext<'_, '_, T, U>> for TWithDiagnosticArray<LoaderResult> {
  fn from(loader_context: LoaderContext<'_, '_, T, U>) -> Self {
    LoaderResult {
      cacheable: loader_context.cacheable,
      file_dependencies: loader_context.file_dependencies,
      context_dependencies: loader_context.context_dependencies,
      missing_dependencies: loader_context.missing_dependencies,
      build_dependencies: loader_context.build_dependencies,
      content: loader_context.content,
      source_map: loader_context.source_map,
      additional_data: loader_context.additional_data,
    }
    .with_diagnostic(loader_context.diagnostic)
  }
}

#[async_trait::async_trait]
pub trait Loader<T, U>: Sync + Send + Debug {
  /// Loader name for module identifier
  fn name(&self) -> &str;

  /// Each loader should expose a `run` fn, which will be called by the loader runner.
  async fn run(&self, loader_context: &mut LoaderContext<'_, '_, T, U>) -> Result<()>;

  fn as_any(&self) -> &dyn std::any::Any;

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub type BoxLoader<T, U> = Arc<dyn Loader<T, U>>;
pub type BoxRunnerPlugin = Box<dyn LoaderRunnerPlugin>;

pub type LoaderRunnerResult = Result<TWithDiagnosticArray<LoaderResult>>;

pub struct LoaderRunner {
  plugins: Vec<BoxRunnerPlugin>,
  resource_data: ResourceData,
}

#[derive(Debug)]
pub struct LoaderRunnerAdditionalContext<'context, T, U> {
  pub compiler: &'context T,
  pub compilation: &'context U,
}

impl LoaderRunner {
  pub fn new(resource_data: ResourceData, plugins: Vec<BoxRunnerPlugin>) -> Self {
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
      content,
      resource: &self.resource_data.resource,
      resource_path: &self.resource_data.resource_path,
      resource_query: self.resource_data.resource_query.as_deref(),
      resource_fragment: self.resource_data.resource_fragment.as_deref(),
      compiler_context: context.compiler,
      compilation_context: context.compilation,
      source_map: None,
      additional_data: None,
      diagnostic: vec![],
    };

    Ok(loader_context)
  }

  pub async fn run<'loader, 'context: 'loader, T, U>(
    &self,
    loaders: impl AsRef<[&'loader dyn Loader<T, U>]>,
    context: &'context LoaderRunnerAdditionalContext<'_, T, U>,
  ) -> LoaderRunnerResult {
    let mut loader_context = self.get_loader_context(context).await?;

    tracing::trace!("Running loaders for resource: {}", loader_context.resource);

    for loader in loaders.as_ref().iter().rev() {
      tracing::trace!("Running loader: {}", loader.name());

      loader.run(&mut loader_context).await?;
    }

    Ok(loader_context.into())
  }
}
