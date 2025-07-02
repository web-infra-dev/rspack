mod entry_dependency;

pub use entry_dependency::*;
use napi::bindgen_prelude::{Either, FromNapiValue};
use rspack_core::DependencyId;

use crate::Dependency;

// DependencyObject is used to uniformly handle Dependency and EntryDependency.
pub struct DependencyObject(Option<DependencyId>);

impl DependencyObject {
  pub fn dependency_id(&self) -> Option<DependencyId> {
    self.0
  }
}

impl FromNapiValue for DependencyObject {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    match Either::<&Dependency, &EntryDependency>::from_napi_value(env, napi_val)? {
      Either::A(dependency) => Ok(DependencyObject(Some(dependency.dependency_id))),
      Either::B(entry_dependency) => Ok(DependencyObject(entry_dependency.dependency_id)),
    }
  }
}
