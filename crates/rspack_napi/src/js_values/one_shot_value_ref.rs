use napi::bindgen_prelude::ToNapiValue;
use napi::sys::{self, napi_env, napi_value};
use napi::Result;

use crate::Ref;

// A RAII (Resource Acquisition Is Initialization) style wrapper around `Ref` that ensures the
// reference is unreferenced when it goes out of scope. This struct maintains a single reference
// count and automatically cleans up when it is dropped.
pub struct OneShotRef {
  env: napi_env,
  inner: Ref,
}

impl OneShotRef {
  pub fn new(env: napi_env, value: napi_value) -> Result<Self> {
    let inner = Ref::new(env, value, 1)?;
    Ok(Self { env, inner })
  }
}

impl Drop for OneShotRef {
  fn drop(&mut self) {
    let _ = self.inner.unref(self.env);
  }
}

impl ToNapiValue for &OneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, &val.inner) }
  }
}

impl ToNapiValue for &mut OneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, &val.inner) }
  }
}
