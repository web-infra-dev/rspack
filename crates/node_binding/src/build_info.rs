use napi::{
  bindgen_prelude::{Object, ToNapiValue, WeakReference},
  Env, JsString, NapiValue,
};
use rspack_core::{Reflector, WeakBindingCell};
use rustc_hash::FxHashMap;

use crate::Module;

// Record<string, Source>
#[napi]
pub struct Assets {
  i: WeakBindingCell<FxHashMap<String, rspack_core::CompilationAsset>>,
}

impl Assets {
  pub fn new(i: WeakBindingCell<FxHashMap<String, rspack_core::CompilationAsset>>) -> Self {
    Self { i }
  }

  fn with_ref<T>(
    &self,
    f: impl FnOnce(&FxHashMap<String, rspack_core::CompilationAsset>) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.i.upgrade() {
      Some(reference) => f(reference.as_ref()),
      None => Err(napi::Error::from_reason(
        "Unable to access assets now. The assets has been dropped by Rust.".to_string(),
      )),
    }
  }
}

#[napi]
impl Assets {
  #[napi]
  pub fn keys(&self, env: &Env) -> napi::Result<Vec<JsString>> {
    self.with_ref(|assets| {
      assets
        .keys()
        .map(|s| env.create_string(s))
        .collect::<napi::Result<Vec<JsString>>>()
    })
  }
}

#[napi]
pub struct BuildInfo {
  module_reference: WeakReference<Module>,
}

impl BuildInfo {
  pub fn new(module_reference: WeakReference<Module>) -> Self {
    Self { module_reference }
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(unsafe { Object::from_raw_unchecked(raw_env, napi_val) })
  }

  fn with_ref<T>(
    &mut self,
    f: impl FnOnce(&dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.module_reference.get_mut() {
      Some(reference) => {
        let (_, module) = reference.as_ref()?;
        f(module)
      }
      None => Err(napi::Error::from_reason(
        "Unable to access buildInfo now. The Module has been garbage collected by JavaScript."
          .to_string(),
      )),
    }
  }
}

#[napi]
impl BuildInfo {
  #[napi(getter, js_name = "_assets", ts_return_type = "Assets")]
  pub fn assets(&mut self) -> napi::Result<Reflector> {
    self.with_ref(|module| Ok(module.build_info().assets.reflector()))
  }
}
