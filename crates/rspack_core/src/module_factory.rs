use std::path::PathBuf;

use rspack_error::{Result, TWithDiagnosticArray};
use rustc_hash::FxHashSet as HashSet;

use crate::{BoxModule, BoxModuleDependency, Context, FactoryMeta, Resolve};

#[derive(Debug)]
pub struct ModuleFactoryCreateData {
  pub resolve_options: Option<Resolve>,
  pub context: Context,
  pub dependency: BoxModuleDependency,
}

#[derive(Debug)]
pub struct ModuleFactoryResult {
  pub module: BoxModule,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub factory_meta: FactoryMeta,
}

impl ModuleFactoryResult {
  pub fn new(module: BoxModule) -> Self {
    Self {
      module,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      factory_meta: Default::default(),
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
}

#[async_trait::async_trait]
pub trait ModuleFactory {
  async fn create(
    mut self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>>;
}

pub type BoxModuleFactory = Box<dyn ModuleFactory>;
