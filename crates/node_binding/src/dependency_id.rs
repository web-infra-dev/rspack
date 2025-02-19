use napi::{
  bindgen_prelude::{ClassInstance, FromNapiValue},
  Either,
};
use rspack_core::DependencyId;

use crate::{JsDependency, JsEntryDependency};

pub struct JsDependencyId(Option<DependencyId>);

impl JsDependencyId {
  pub fn raw(&self) -> Option<&DependencyId> {
    self.0.as_ref()
  }
}

impl FromNapiValue for JsDependencyId {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    // Check subclass first, then parent class
    let union =
      <Either<ClassInstance<JsEntryDependency>, ClassInstance<JsDependency>> as FromNapiValue>::from_napi_value(
        env, napi_val,
      )?;

    let dependency_id = match union {
      Either::B(dependency) => Some(dependency.dependency_id),
      Either::A(entry_dependency) => entry_dependency
        .parent
        .as_ref()
        .map(|dep| dep.dependency_id),
    };

    Ok(JsDependencyId(dependency_id))
  }
}
