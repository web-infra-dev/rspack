use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ModuleDependency, ModuleFactoryCreateData,
};

#[derive(Debug, Clone)]
pub(crate) struct LazyCompilationDependency {
  id: DependencyId,
  pub original_module_create_data: ModuleFactoryCreateData,
  request: String,
}

impl LazyCompilationDependency {
  pub fn new(original_module_create_data: ModuleFactoryCreateData) -> Self {
    let dep = original_module_create_data
      .dependency
      .as_module_dependency()
      .expect("LazyCompilation: should convert to module dependency");
    let request = format!("{}?lazy-compilation-proxy-dep", dep.request());

    Self {
      id: DependencyId::new(),
      original_module_create_data,
      request,
    }
  }
}

impl ModuleDependency for LazyCompilationDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsDependencyTemplate for LazyCompilationDependency {}
impl AsContextDependency for LazyCompilationDependency {}

impl Dependency for LazyCompilationDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "lazy compilation dependency"
  }

  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::LazyImport
  }
}
