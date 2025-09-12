use std::{fmt::Debug, sync::Arc};

use rspack_error::{Diagnostic, Result};
use rspack_paths::{ArcPath, ArcPathSet};

use crate::{
  BoxDependency, BoxModule, CompilationId, CompilerId, CompilerOptions, Context, ModuleIdentifier,
  ModuleLayer, Resolve, ResolverFactory,
};

#[derive(Debug, Clone)]
pub struct ModuleFactoryCreateData {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub resolve_options: Option<Arc<Resolve>>,
  pub options: Arc<CompilerOptions>,
  pub request: String,
  pub context: Context,
  pub dependencies: Vec<BoxDependency>,
  pub issuer: Option<Box<str>>,
  pub issuer_identifier: Option<ModuleIdentifier>,
  pub issuer_layer: Option<ModuleLayer>,
  pub resolver_factory: Arc<ResolverFactory>,

  pub file_dependencies: ArcPathSet,
  pub context_dependencies: ArcPathSet,
  pub missing_dependencies: ArcPathSet,
  pub diagnostics: Vec<Diagnostic>,
}

impl ModuleFactoryCreateData {
  pub fn add_file_dependency<F: Into<ArcPath>>(&mut self, file: F) {
    self.file_dependencies.insert(file.into());
  }

  pub fn add_file_dependencies<F: Into<ArcPath>>(&mut self, files: impl IntoIterator<Item = F>) {
    self
      .file_dependencies
      .extend(files.into_iter().map(Into::into));
  }

  pub fn add_context_dependency<F: Into<ArcPath>>(&mut self, context: F) {
    self.context_dependencies.insert(context.into());
  }

  pub fn add_context_dependencies<F: Into<ArcPath>>(
    &mut self,
    contexts: impl IntoIterator<Item = F>,
  ) {
    self
      .context_dependencies
      .extend(contexts.into_iter().map(Into::into));
  }

  pub fn add_missing_dependency<F: Into<ArcPath>>(&mut self, missing: F) {
    self.missing_dependencies.insert(missing.into());
  }

  pub fn add_missing_dependencies<F: Into<ArcPath>>(
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

  pub fn module(mut self, module: Option<BoxModule>) -> Self {
    self.module = module;
    self
  }
}

#[async_trait::async_trait]
pub trait ModuleFactory: Debug + Sync + Send {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult>;
}
