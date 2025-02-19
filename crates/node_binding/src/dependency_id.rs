use napi::bindgen_prelude::FromNapiValue;
use rspack_core::DependencyId;

use crate::JsDependency;

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
    let js_dependency = <&JsDependency as FromNapiValue>::from_napi_value(env, napi_val)?;
    let dependency_id = js_dependency
      .0
      .as_ref()
      .map(|binding| binding.dependency_id);
    Ok(JsDependencyId(dependency_id))
  }
}
