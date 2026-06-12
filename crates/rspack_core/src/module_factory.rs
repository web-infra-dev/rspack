use std::{
  any::{Any, TypeId},
  fmt::Debug,
  sync::Arc,
};

use rspack_error::{Diagnostic, Result};
use rspack_paths::{ArcPath, ArcPathSet};

use crate::{
  BoxDependency, BoxModule, CompilationId, CompilerId, CompilerOptions, Context,
  ContextModuleFactory, IgnoreErrorModuleFactory, ModuleIdentifier, ModuleLayer,
  NormalModuleFactory, Resolve, ResolverFactory, SelfModuleFactory,
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

#[derive(Debug, Clone)]
pub enum ModuleFactoryKind {
  Normal(Arc<NormalModuleFactory>),
  Context(Arc<ContextModuleFactory>),
  IgnoreError(Arc<IgnoreErrorModuleFactory>),
  SelfModule(Arc<SelfModuleFactory>),
  Custom(Arc<dyn ModuleFactory>),
}

impl ModuleFactoryKind {
  pub fn normal(factory: Arc<NormalModuleFactory>) -> Self {
    Self::Normal(factory)
  }

  pub fn context(factory: Arc<ContextModuleFactory>) -> Self {
    Self::Context(factory)
  }

  pub fn ignore_error(factory: Arc<IgnoreErrorModuleFactory>) -> Self {
    Self::IgnoreError(factory)
  }

  pub fn self_module(factory: Arc<SelfModuleFactory>) -> Self {
    Self::SelfModule(factory)
  }

  pub fn custom(factory: Arc<dyn ModuleFactory>) -> Self {
    Self::Custom(factory)
  }

  pub async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    match self {
      Self::Normal(factory) => factory.create(data).await,
      Self::Context(factory) => factory.create(data).await,
      Self::IgnoreError(factory) => factory.create(data).await,
      Self::SelfModule(factory) => factory.create(data).await,
      Self::Custom(factory) => factory.create(data).await,
    }
  }
}

pub trait IntoModuleFactoryKind {
  fn into_module_factory_kind(self) -> ModuleFactoryKind;
}

impl IntoModuleFactoryKind for ModuleFactoryKind {
  fn into_module_factory_kind(self) -> ModuleFactoryKind {
    self
  }
}

impl IntoModuleFactoryKind for Arc<dyn ModuleFactory> {
  fn into_module_factory_kind(self) -> ModuleFactoryKind {
    ModuleFactoryKind::Custom(self)
  }
}

impl<T> IntoModuleFactoryKind for Arc<T>
where
  T: ModuleFactory + Any + 'static,
{
  fn into_module_factory_kind(self) -> ModuleFactoryKind {
    if TypeId::of::<T>() == TypeId::of::<NormalModuleFactory>() {
      let factory = self as Arc<dyn Any + Send + Sync>;
      return ModuleFactoryKind::Normal(
        Arc::downcast::<NormalModuleFactory>(factory)
          .expect("module factory type id should match NormalModuleFactory"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<ContextModuleFactory>() {
      let factory = self as Arc<dyn Any + Send + Sync>;
      return ModuleFactoryKind::Context(
        Arc::downcast::<ContextModuleFactory>(factory)
          .expect("module factory type id should match ContextModuleFactory"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<IgnoreErrorModuleFactory>() {
      let factory = self as Arc<dyn Any + Send + Sync>;
      return ModuleFactoryKind::IgnoreError(
        Arc::downcast::<IgnoreErrorModuleFactory>(factory)
          .expect("module factory type id should match IgnoreErrorModuleFactory"),
      );
    }

    if TypeId::of::<T>() == TypeId::of::<SelfModuleFactory>() {
      let factory = self as Arc<dyn Any + Send + Sync>;
      return ModuleFactoryKind::SelfModule(
        Arc::downcast::<SelfModuleFactory>(factory)
          .expect("module factory type id should match SelfModuleFactory"),
      );
    }

    ModuleFactoryKind::Custom(self)
  }
}
