use napi::{bindgen_prelude::WeakReference, Env, JsString};
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
      None => Err(napi::Error::from_reason(format!(
        "Unable to access assets now. The assets has been dropped by Rust."
      ))),
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

  fn with_ref<T>(
    &mut self,
    f: impl FnOnce(&dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.module_reference.get_mut() {
      Some(reference) => {
        let (_, module) = reference.as_ref()?;
        f(module)
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to access buildInfo now. The Module has been garbage collected by JavaScript."
      ))),
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
