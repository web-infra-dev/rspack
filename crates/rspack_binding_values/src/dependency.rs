use napi_derive::napi;
use rspack_core::{BoxDependency, DependencyId};

// JsDependency allows JS-side access to a Dependency instance that has already
// been processed and stored in the Compilation.
#[napi]
pub struct JsDependency(&'static BoxDependency);

impl JsDependency {
  pub(crate) fn new(dependency: &BoxDependency) -> Self {
    // SAFETY:
    // The lifetime of the &mut BoxDependency reference is extended to 'static.
    // Accessing fields and methods on the Rust object from the JS side after the Rust object's
    // lifetime has ended is undefined behavior, which we currently disregard.
    let dependency =
      unsafe { std::mem::transmute::<&BoxDependency, &'static BoxDependency>(dependency) };
    Self(dependency)
  }

  pub(crate) fn id(&self) -> &DependencyId {
    self.0.id()
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
  pub fn critical(&self) -> bool {
    match self.0.as_context_dependency() {
      Some(dep) => dep.critical().is_some(),
      None => false,
    }
  }
}

// JsDependency represents a Dependency instance that is currently being processed.
// It is in the make stage and has not yet been added to the Compilation.
#[napi]
pub struct JsDependencyMut(&'static mut BoxDependency);

impl JsDependencyMut {
  pub(crate) fn new(dependency: &mut BoxDependency) -> Self {
    // SAFETY:
    // The lifetime of the &mut BoxDependency reference is extended to 'static.
    // Accessing fields and methods on the Rust object from the JS side after the Rust object's
    // lifetime has ended is undefined behavior, which we currently disregard.
    let dependency =
      unsafe { std::mem::transmute::<&mut BoxDependency, &'static mut BoxDependency>(dependency) };
    Self(dependency)
  }
}

#[napi]
impl JsDependencyMut {
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
  pub fn critical(&self) -> bool {
    match self.0.as_context_dependency() {
      Some(dep) => dep.critical().is_some(),
      None => false,
    }
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) {
    if let Some(dep) = self.0.as_context_dependency_mut() {
      let critical = dep.critical_mut();
      if !val {
        *critical = None;
      }
    }
  }
}
