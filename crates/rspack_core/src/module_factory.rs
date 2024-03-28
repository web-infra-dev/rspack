use std::{fmt::Debug, path::PathBuf};

use rspack_error::{Diagnostic, Result};
use rustc_hash::FxHashSet as HashSet;
use sugar_path::SugarPathBuf;

use crate::{BoxDependency, BoxModule, Context, FactoryMeta, ModuleIdentifier, Resolve};

#[derive(Debug, Clone)]
pub struct ModuleFactoryCreateData {
  pub resolve_options: Option<Box<Resolve>>,
  pub context: Context,
  pub dependency: BoxDependency,
  pub issuer: Option<Box<str>>,
  pub issuer_identifier: Option<ModuleIdentifier>,

  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub diagnostics: Vec<Diagnostic>,
}

impl ModuleFactoryCreateData {
  pub fn add_file_dependency(&mut self, file: PathBuf) {
    if file.is_absolute() {
      self.file_dependencies.insert(file.into_normalize());
    }
  }

  pub fn add_file_dependencies(&mut self, files: impl IntoIterator<Item = PathBuf>) {
    self
      .file_dependencies
      .extend(files.into_iter().map(|x| x.into_normalize()));
  }

  pub fn add_context_dependency(&mut self, context: PathBuf) {
    self.context_dependencies.insert(context.into_normalize());
  }

  pub fn add_context_dependencies(&mut self, contexts: impl IntoIterator<Item = PathBuf>) {
    self
      .context_dependencies
      .extend(contexts.into_iter().map(|x| x.into_normalize()));
  }

  pub fn add_missing_dependency(&mut self, missing: PathBuf) {
    self.missing_dependencies.insert(missing.into_normalize());
  }

  pub fn add_missing_dependencies(&mut self, missing: impl IntoIterator<Item = PathBuf>) {
    self
      .missing_dependencies
      .extend(missing.into_iter().map(|x| x.into_normalize()));
  }
}

#[derive(Debug, Default)]
pub struct ModuleFactoryResult {
  pub module: Option<BoxModule>,
  pub factory_meta: FactoryMeta,
  pub from_cache: bool,
}

impl ModuleFactoryResult {
  pub fn new_with_module(module: BoxModule) -> Self {
    Self {
      module: Some(module),
      factory_meta: Default::default(),
      from_cache: false,
    }
  }

  pub fn module(mut self, module: Option<BoxModule>) -> Self {
    self.module = module;
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
pub trait ModuleFactory: Debug + Sync + Send {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult>;
}
