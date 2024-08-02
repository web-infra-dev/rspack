use napi_derive::napi;
use rspack_core::{Compilation, DependencyId};

#[napi]
pub struct JsDependency {
  pub(crate) dependency_id: DependencyId,
  pub(crate) compilation: &'static Compilation,
}

impl JsDependency {
  pub(crate) fn new(dependency_id: DependencyId, compilation: &'static Compilation) -> Self {
    Self {
      dependency_id,
      compilation,
    }
  }
}

#[napi]
impl JsDependency {
  #[napi(getter)]
  pub fn get_type(&self) -> &str {
    let module_graph = self.compilation.get_module_graph();
    let dep = module_graph
      .dependency_by_id(&self.dependency_id)
      .unwrap_or_else(|| panic!("Failed to get dependency by id = {:?}", &self.dependency_id));
    dep.dependency_type().as_str()
  }

  #[napi(getter)]
  pub fn category(&self) -> &str {
    let module_graph = self.compilation.get_module_graph();
    let dep = module_graph
      .dependency_by_id(&self.dependency_id)
      .unwrap_or_else(|| panic!("Failed to get dependency by id = {:?}", &self.dependency_id));
    dep.category().as_str()
  }
}
