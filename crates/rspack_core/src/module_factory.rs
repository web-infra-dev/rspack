use std::{fmt::Debug, sync::Arc};

use rspack_error::{Diagnostic, Result};
use rspack_paths::{InternedPath, InternedPathSet};

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

  pub file_dependencies: InternedPathSet,
  pub context_dependencies: InternedPathSet,
  pub missing_dependencies: InternedPathSet,
  pub diagnostics: Vec<Diagnostic>,
}

impl ModuleFactoryCreateData {
  pub fn add_file_dependencies<F: Into<InternedPath>>(&mut self, files: impl IntoIterator<Item = F>) {
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
pub trait ModuleFactory: Debug + Sync + Send {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult>;
}
