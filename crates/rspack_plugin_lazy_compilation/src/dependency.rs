use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency,
};
use rspack_error::Diagnostic;
use rspack_paths::ArcPathSet;

#[cacheable]
#[derive(Debug, Clone)]
pub struct DependencyOptions {
  pub request: String,

  pub file_dependencies: ArcPathSet,
  pub context_dependencies: ArcPathSet,
  pub missing_dependencies: ArcPathSet,
  pub diagnostics: Vec<Diagnostic>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct LazyCompilationDependency {
  id: DependencyId,
  factorize_info: FactorizeInfo,
  options: DependencyOptions,
}

impl LazyCompilationDependency {
  pub fn new(options: DependencyOptions) -> Self {
    Self {
      id: DependencyId::new(),
      factorize_info: Default::default(),
      options,
    }
  }

  pub fn options(&self) -> &DependencyOptions {
    &self.options
  }
}

#[cacheable_dyn]
impl ModuleDependency for LazyCompilationDependency {
  fn request(&self) -> &str {
    &self.options.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyCodeGeneration for LazyCompilationDependency {}
impl AsContextDependency for LazyCompilationDependency {}

#[cacheable_dyn]
impl Dependency for LazyCompilationDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::LazyImport
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}
