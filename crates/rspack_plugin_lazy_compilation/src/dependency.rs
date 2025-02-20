use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, FactorizeInfo, ModuleDependency, ModuleFactoryCreateData,
};

#[cacheable]
#[derive(Debug, Clone)]
pub(crate) struct LazyCompilationDependency {
  id: DependencyId,
  pub original_module_create_data: ModuleFactoryCreateData,
  request: String,
  factorize_info: FactorizeInfo,
}

impl LazyCompilationDependency {
  pub fn new(original_module_create_data: ModuleFactoryCreateData) -> Self {
    let dep = original_module_create_data.dependencies[0]
      .as_module_dependency()
      .expect("LazyCompilation: should convert to module dependency");
    let request = dep.request().to_string();

    Self {
      id: DependencyId::new(),
      original_module_create_data,
      request,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl ModuleDependency for LazyCompilationDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsDependencyTemplate for LazyCompilationDependency {}
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
