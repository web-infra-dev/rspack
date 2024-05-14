use std::{fmt::Debug, path::PathBuf};

use rspack_error::{Diagnostic, Result};
use rustc_hash::FxHashSet as HashSet;
use sugar_path::SugarPath;

use crate::{BoxDependency, BoxModule, Context, ModuleIdentifier, Resolve};

#[derive(Debug)]
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
  pub fn request(&self) -> Option<&str> {
    self
      .dependency
      .as_module_dependency()
      .map(|d| d.request())
      .or_else(|| self.dependency.as_context_dependency().map(|d| d.request()))
  }

  pub fn add_file_dependency(&mut self, file: PathBuf) {
    if file.is_absolute() {
      self.file_dependencies.insert(file.normalize());
    }
  }

  pub fn add_file_dependencies(&mut self, files: impl IntoIterator<Item = PathBuf>) {
    self
      .file_dependencies
      .extend(files.into_iter().map(|x| x.normalize()));
  }

  pub fn add_context_dependency(&mut self, context: PathBuf) {
    self.context_dependencies.insert(context.normalize());
  }

  pub fn add_context_dependencies(&mut self, contexts: impl IntoIterator<Item = PathBuf>) {
    self
      .context_dependencies
      .extend(contexts.into_iter().map(|x| x.normalize()));
  }

  pub fn add_missing_dependency(&mut self, missing: PathBuf) {
    self.missing_dependencies.insert(missing.normalize());
  }

  pub fn add_missing_dependencies(&mut self, missing: impl IntoIterator<Item = PathBuf>) {
    self
      .missing_dependencies
      .extend(missing.into_iter().map(|x| x.normalize()));
  }
}

#[derive(Debug, Default)]
pub struct ModuleFactoryResult {
  pub module: Option<BoxModule>,
  pub from_cache: bool,
}

impl ModuleFactoryResult {
  pub fn new_with_module(module: BoxModule) -> Self {
    Self {
      module: Some(module),
      from_cache: false,
    }
  }

  pub fn module(mut self, module: Option<BoxModule>) -> Self {
    self.module = module;
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
