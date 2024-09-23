use napi_derive::napi;
use rspack_core::{
  BoxDependency, Compilation, ContextDependency, Dependency, DependencyId, ModuleDependency,
  ModuleGraph,
};

// JsCompiledDependency allows JS-side access to a Dependency instance that has already
// been processed and stored in the Compilation.
#[napi]
pub struct JsCompiledDependency {
  pub(crate) dependency_id: DependencyId,
  pub(crate) module_graph: ModuleGraph<'static>,
}

impl JsCompiledDependency {
  pub(crate) fn new(dependency_id: DependencyId, compilation: &'static Compilation) -> Self {
    let module_graph = compilation.get_module_graph();
    Self {
      dependency_id,
      module_graph,
    }
  }

  fn dependency<'a>(&self) -> &dyn Dependency {
    self
      .module_graph
      .dependency_by_id(&self.dependency_id)
      .unwrap_or_else(|| panic!("Failed to get dependency by id = {:?}", &self.dependency_id))
      .as_ref()
  }

  fn module_dependency<'a>(&self) -> Option<&dyn ModuleDependency> {
    self.dependency().as_module_dependency()
  }

  fn context_dependency<'a>(&self) -> Option<&dyn ContextDependency> {
    self.dependency().as_context_dependency()
  }
}

#[napi]
impl JsCompiledDependency {
  #[napi(getter)]
  pub fn get_type(&self) -> &str {
    self.dependency().dependency_type().as_str()
  }

  #[napi(getter)]
  pub fn category(&self) -> &str {
    self.dependency().category().as_str()
  }

  #[napi(getter)]
  pub fn request(&self) -> napi::Either<&str, ()> {
    match self.module_dependency() {
      Some(dep) => napi::Either::A(dep.request()),
      None => napi::Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn critical(&self) -> napi::Either<bool, ()> {
    match self.context_dependency() {
      Some(dep) => napi::Either::A(dep.critical().is_some()),
      None => napi::Either::B(()),
    }
  }
}

// JsDependency represents a Dependency instance that is currently being processed.
// It is in the make stage and has not yet been added to the Compilation.
#[napi]
pub struct JsDependency(&'static mut BoxDependency);

impl JsDependency {
  pub(crate) fn new(dependency: &mut BoxDependency) -> Self {
    // SAFETY:
    // The lifetime of the &mut BoxDependency reference is extended to 'static.
    // This is safe because the JS side will guarantee that the JsDependency instance's
    // lifetime is properly managed and restricted.
    let dependency =
      unsafe { std::mem::transmute::<&mut BoxDependency, &'static mut BoxDependency>(dependency) };
    Self(dependency)
  }
}

#[napi]
impl JsDependency {
  #[napi(getter)]
  pub fn get_type(&self) -> &str {
    self.0.dependency_type().as_str()
  }

  #[napi(getter)]
  pub fn category(&self) -> &str {
    self.0.category().as_str()
  }

  #[napi(getter)]
  pub fn request(&self) -> napi::Either<&str, ()> {
    match self.0.as_module_dependency() {
      Some(dep) => napi::Either::A(dep.request()),
      None => napi::Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn critical(&self) -> napi::Either<bool, ()> {
    match self.0.as_context_dependency() {
      Some(dep) => napi::Either::A(dep.critical().is_some()),
      None => napi::Either::B(()),
    }
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) {
    match self.0.as_context_dependency_mut() {
      Some(dep) => {
        let critical = dep.critical_mut();
        if !val {
          *critical = None;
        }
      }
      None => (),
    }
  }
}
