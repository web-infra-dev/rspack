use std::{fmt::Debug, sync::Arc};

use rspack_error::{Diagnostic, Result};
use rspack_paths::{InternedPath, InternedPathSet};

use crate::{
  BoxModule, BuildContext, Context, DependencyRef, ModuleIdentifier, ModuleLayer, Resolve,
};

#[derive(Debug)]
pub struct ModuleFactoryCreateData {
  pub build_context: Arc<BuildContext>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub request: String,
  pub context: Context,
  pub dependencies: Vec<DependencyRef>,
  pub issuer: Option<Box<str>>,
  pub issuer_identifier: Option<ModuleIdentifier>,
  pub issuer_layer: Option<ModuleLayer>,

  pub file_dependencies: InternedPathSet,
  pub context_dependencies: InternedPathSet,
  pub missing_dependencies: InternedPathSet,
  pub diagnostics: Vec<Diagnostic>,
}

impl ModuleFactoryCreateData {
  /// Creates factory inputs from a build context and a non-empty dependency group.
  pub fn new(
    build_context: Arc<BuildContext>,
    resolve_options: Option<Arc<Resolve>>,
    original_module_context: Option<&Context>,
    dependencies: Vec<DependencyRef>,
    issuer: Option<Box<str>>,
    issuer_identifier: Option<ModuleIdentifier>,
    issuer_layer: Option<ModuleLayer>,
  ) -> Self {
    let dependency = &dependencies[0];
    let context = if let Some(context) = dependency.get_context()
      && !context.is_empty()
    {
      context
    } else if let Some(context) = dependency
      .as_context_dependency()
      .and_then(|dependency| crate::ContextDependency::get_context(dependency))
      && !context.is_empty()
    {
      context
    } else if let Some(context) = original_module_context
      && !context.is_empty()
    {
      context
    } else {
      &build_context.compiler_options.context
    }
    .into();
    let issuer_layer = dependency.get_layer().or(issuer_layer.as_ref()).cloned();
    let request = dependency
      .as_module_dependency()
      .map(|dependency| dependency.request().to_string())
      .or_else(|| {
        dependency
          .as_context_dependency()
          .map(|dependency| dependency.request().to_string())
      })
      .unwrap_or_default();

    Self {
      build_context,
      resolve_options,
      request,
      context,
      dependencies,
      issuer,
      issuer_identifier,
      issuer_layer,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      diagnostics: Default::default(),
    }
  }

  pub fn add_file_dependencies<F: Into<InternedPath>>(
    &mut self,
    files: impl IntoIterator<Item = F>,
  ) {
    self
      .file_dependencies
      .extend(files.into_iter().map(Into::into));
  }

  pub fn add_missing_dependencies<F: Into<InternedPath>>(
    &mut self,
    missing: impl IntoIterator<Item = F>,
  ) {
    self
      .missing_dependencies
      .extend(missing.into_iter().map(Into::into));
  }
}

#[derive(Debug, Default)]
pub struct ModuleFactoryResult {
  pub module: Option<BoxModule>,
}

impl ModuleFactoryResult {
  pub fn new_with_module(module: BoxModule) -> Self {
    Self {
      module: Some(module),
    }
  }
}

#[async_trait::async_trait]
pub trait ModuleFactory: rspack_util::MaybeAllocative + Debug + Sync + Send {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult>;
}
