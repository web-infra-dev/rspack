use std::{fmt::Debug, path::PathBuf};

use rspack_error::{Diagnosable, Diagnostic, Result};
use rustc_hash::FxHashSet as HashSet;

use crate::{BoxDependency, BoxModule, Context, FactoryMeta, ModuleIdentifier, Resolve};

#[derive(Debug)]
pub struct ModuleFactoryCreateData {
  pub resolve_options: Option<Box<Resolve>>,
  pub context: Context,
  pub dependency: BoxDependency,
  pub issuer: Option<Box<str>>,
  pub issuer_identifier: Option<ModuleIdentifier>,
}

#[derive(Debug)]
pub struct ModuleFactoryResult {
  pub module: BoxModule,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub factory_meta: FactoryMeta,
  pub from_cache: bool,
}

impl ModuleFactoryResult {
  pub fn new(module: BoxModule) -> Self {
    Self {
      module,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      factory_meta: Default::default(),
      from_cache: false,
    }
  }

  pub fn file_dependency(mut self, file: PathBuf) -> Self {
    if file.is_absolute() {
      self.file_dependencies.insert(file);
    }
    self
  }

  pub fn file_dependencies(mut self, files: impl IntoIterator<Item = PathBuf>) -> Self {
    self.file_dependencies.extend(files);
    self
  }

  pub fn context_dependency(mut self, context: PathBuf) -> Self {
    self.context_dependencies.insert(context);
    self
  }

  pub fn context_dependencies(mut self, contexts: impl IntoIterator<Item = PathBuf>) -> Self {
    self.context_dependencies.extend(contexts);
    self
  }

  pub fn missing_dependency(mut self, missing: PathBuf) -> Self {
    self.missing_dependencies.insert(missing);
    self
  }

  pub fn missing_dependencies(mut self, missing: impl IntoIterator<Item = PathBuf>) -> Self {
    self.missing_dependencies.extend(missing);
    self
  }

  pub fn factory_meta(mut self, factory_meta: FactoryMeta) -> Self {
    self.factory_meta = factory_meta;
    self
  }

  pub fn from_cache(mut self, from_cache: bool) -> Self {
    self.from_cache = from_cache;
    self
  }
}

#[async_trait::async_trait]
pub trait ModuleFactory: Debug + Sync + Send + Diagnosable {
  async fn create(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<(ModuleFactoryResult, Vec<Diagnostic>)>;
}
