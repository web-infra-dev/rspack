use napi_derive::napi;
use rspack_core::{Compilation, Dependency, DependencyId, ModuleDependency, ModuleGraph};

#[napi]
pub struct DependencyDTO {
  pub(crate) dependency_id: DependencyId,
  pub(crate) compilation: &'static Compilation,
}

impl DependencyDTO {
  pub(crate) fn new(dependency_id: DependencyId, compilation: &'static Compilation) -> Self {
    Self {
      dependency_id,
      compilation,
    }
  }

  fn dependency<'a>(&self, module_graph: &'a ModuleGraph) -> &'a dyn Dependency {
    module_graph
      .dependency_by_id(&self.dependency_id)
      .unwrap_or_else(|| panic!("Failed to get dependency by id = {:?}", &self.dependency_id))
      .as_ref()
  }

  fn module_dependency<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Option<&'a dyn ModuleDependency> {
    self.dependency(module_graph).as_module_dependency()
  }
}

#[napi]
impl DependencyDTO {
  #[napi(getter)]
  pub fn get_type(&self) -> &str {
    let module_graph = self.compilation.get_module_graph();
    let dep = self.dependency(&module_graph);
    dep.dependency_type().as_str()
  }

  #[napi(getter)]
  pub fn category(&self) -> &str {
    let module_graph = self.compilation.get_module_graph();
    let dep = self.dependency(&module_graph);
    dep.category().as_str()
  }

  #[napi(getter)]
  pub fn request(&self) -> napi::Either<String, ()> {
    let module_graph = self.compilation.get_module_graph();
    match self.module_dependency(&module_graph) {
      Some(dep) => napi::Either::A(dep.request().to_string()),
      None => napi::Either::B(()),
    }
  }
}
