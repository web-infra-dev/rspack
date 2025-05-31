use napi::{
  bindgen_prelude::{Object, ToNapiValue, WeakReference},
  Env, JsString,
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
  pub fn keys<'a>(&self, env: &'a Env) -> napi::Result<Vec<JsString<'a>>> {
    self.with_ref(|assets| {
      assets
        .keys()
        .map(|s| env.create_string(s))
        .collect::<napi::Result<Vec<JsString>>>()
    })
  }
}

#[napi]
pub struct KnownBuildInfo {
  module_reference: WeakReference<Module>,
}

impl KnownBuildInfo {
  pub fn new(module_reference: WeakReference<Module>) -> Self {
    Self { module_reference }
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(Object::from_raw(raw_env, napi_val))
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
impl KnownBuildInfo {
  #[napi(getter, js_name = "_assets", ts_return_type = "Assets")]
  pub fn assets(&mut self) -> napi::Result<Reflector> {
    self.with_ref(|module| Ok(module.build_info().assets.reflector()))
  }

  #[napi(getter, js_name = "_fileDependencies")]
  pub fn file_dependencies<'a>(&mut self, env: &'a Env) -> napi::Result<Vec<JsString<'a>>> {
    self.with_ref(|module| {
      module
        .build_info()
        .file_dependencies
        .iter()
        .map(|dependency| env.create_string(dependency.to_string_lossy().as_ref()))
        .collect::<napi::Result<Vec<JsString>>>()
    })
  }

  #[napi(getter, js_name = "_contextDependencies")]
  pub fn context_dependencies<'a>(&mut self, env: &'a Env) -> napi::Result<Vec<JsString<'a>>> {
    self.with_ref(|module| {
      module
        .build_info()
        .context_dependencies
        .iter()
        .map(|dependency| env.create_string(dependency.to_string_lossy().as_ref()))
        .collect::<napi::Result<Vec<JsString>>>()
    })
  }

  #[napi(getter, js_name = "_missingDependencies")]
  pub fn missing_dependencies<'a>(&mut self, env: &'a Env) -> napi::Result<Vec<JsString<'a>>> {
    self.with_ref(|module| {
      module
        .build_info()
        .missing_dependencies
        .iter()
        .map(|dependency| env.create_string(dependency.to_string_lossy().as_ref()))
        .collect::<napi::Result<Vec<JsString>>>()
    })
  }

  #[napi(getter, js_name = "_buildDependencies")]
  pub fn build_dependencies<'a>(&mut self, env: &'a Env) -> napi::Result<Vec<JsString<'a>>> {
    self.with_ref(|module| {
      module
        .build_info()
        .build_dependencies
        .iter()
        .map(|dependency| env.create_string(dependency.to_string_lossy().as_ref()))
        .collect::<napi::Result<Vec<JsString>>>()
    })
  }
}
