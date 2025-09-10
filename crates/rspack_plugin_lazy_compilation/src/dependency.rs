use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency,
};
use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

#[cacheable]
#[derive(Debug, Clone)]
pub struct DependencyOptions {
  pub request: String,

  pub file_dependencies: HashSet<ArcPath>,
  pub context_dependencies: HashSet<ArcPath>,
  pub missing_dependencies: HashSet<ArcPath>,
  pub diagnostics: Vec<Diagnostic>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct LazyCompilationDependency {
  id: DependencyId,
  options: DependencyOptions,
}

impl LazyCompilationDependency {
  pub fn new(options: DependencyOptions) -> Self {
    Self {
      id: DependencyId::new(),
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
