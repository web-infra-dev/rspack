use std::{cell::RefCell, sync::LazyLock};

use napi::{
  Env, JsString, JsValue, Property, PropertyAttributes, Unknown,
  bindgen_prelude::{
    Array, FromNapiMutRef, FromNapiValue, JsObjectValue, Object, ToNapiValue, WeakReference,
  },
};
use rspack_core::WeakBindingCell;
use rspack_napi::unknown_to_json_value;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{define_symbols, module::Module};

define_symbols! {
  BUILD_INFO_ASSETS_SYMBOL => "BUILD_INFO_ASSETS_SYMBOL",
  BUILD_INFO_FILE_DEPENDENCIES_SYMBOL => "BUILD_INFO_FILE_DEPENDENCIES_SYMBOL",
  BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL => "BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL",
  BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL => "BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL",
  BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL => "BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL",
  COMMIT_CUSTOM_FIELDS_SYMBOL => "COMMIT_CUSTOM_FIELDS_SYMBOL",
}

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
        "Unable to access assets. The assets has been dropped by Rust.".to_string(),
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

static KNOWN_BUILD_INFO_FIELD_NAMES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
  FxHashSet::from_iter(vec![
    "assets",
    "fileDependencies",
    "contextDependencies",
    "missingDependencies",
    "buildDependencies",
  ])
});

#[napi]
pub struct KnownBuildInfo {
  module_reference: WeakReference<Module>,
}

impl KnownBuildInfo {
  pub fn new(module_reference: WeakReference<Module>) -> Self {
    Self { module_reference }
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object<'_>> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(Object::from_raw(raw_env, napi_val))
  }

  pub fn with_ref<T>(
    &mut self,
    f: impl FnOnce(&dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.module_reference.get_mut() {
      Some(reference) => {
        let (_, module) = reference.as_ref()?;
        f(module)
      }
      None => Err(napi::Error::from_reason(
        "Unable to access buildInfo. The Module has been garbage collected by JavaScript."
          .to_string(),
      )),
    }
  }

  pub fn with_mut<T>(
    &mut self,
    f: impl FnOnce(&mut dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.module_reference.get_mut() {
      Some(reference) => {
        let module = reference.as_mut()?;
        f(module)
      }
      None => Err(napi::Error::from_reason(
        "Unable to access buildInfo. The Module has been garbage collected by JavaScript."
          .to_string(),
      )),
    }
  }
}

// KnownBuildInfo & Record<string, any>
pub struct BuildInfo {
  module_reference: WeakReference<Module>,
}

impl BuildInfo {
  pub fn new(module_reference: WeakReference<Module>) -> Self {
    Self { module_reference }
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object<'_>> {
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
        "Unable to access buildInfo. The Module has been garbage collected by JavaScript."
          .to_string(),
      )),
    }
  }
}

fn create_known_private_properties(env: &Env, properties: &mut Vec<Property>) -> napi::Result<()> {
  BUILD_INFO_ASSETS_SYMBOL.with(|once_cell| {
    #[allow(clippy::unwrap_used)]
    let symbol = once_cell.get().unwrap();
    properties.push(
      Property::new()
        .with_name(env, symbol)?
        .with_getter_closure(|env, this| {
          let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
          wrapped_value.with_ref(|module| Ok(module.build_info().assets.reflector()))
        })
        .with_property_attributes(PropertyAttributes::Configurable),
    );
    Ok::<(), napi::Error>(())
  })?;

  BUILD_INFO_FILE_DEPENDENCIES_SYMBOL.with(|once_cell| {
    #[allow(clippy::unwrap_used)]
    let symbol = once_cell.get().unwrap();
    properties.push(
      Property::new()
        .with_name(env, symbol)?
        .with_getter_closure(|env, this| {
          let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
          let env_ref = &env;
          let result = wrapped_value.with_ref(|module| {
            module
              .build_info()
              .file_dependencies
              .iter()
              .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
              .collect::<napi::Result<Vec<JsString>>>()
          });
          unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
        })
        .with_property_attributes(PropertyAttributes::Configurable),
    );
    Ok::<(), napi::Error>(())
  })?;

  BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL.with(|once_cell| {
    #[allow(clippy::unwrap_used)]
    let symbol = once_cell.get().unwrap();
    properties.push(
      Property::new()
        .with_name(env, symbol)?
        .with_getter_closure(|env, this| {
          let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
          let env_ref = &env;
          let result = wrapped_value.with_ref(|module| {
            module
              .build_info()
              .context_dependencies
              .iter()
              .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
              .collect::<napi::Result<Vec<JsString>>>()
          });
          unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
        })
        .with_property_attributes(PropertyAttributes::Configurable),
    );
    Ok::<(), napi::Error>(())
  })?;

  BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL.with(|once_cell| {
    #[allow(clippy::unwrap_used)]
    let symbol = once_cell.get().unwrap();
    properties.push(
      Property::new()
        .with_name(env, symbol)?
        .with_getter_closure(|env, this| {
          let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
          let env_ref = &env;
          let result = wrapped_value.with_ref(|module| {
            module
              .build_info()
              .missing_dependencies
              .iter()
              .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
              .collect::<napi::Result<Vec<JsString>>>()
          });
          unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
        })
        .with_property_attributes(PropertyAttributes::Configurable),
    );
    Ok::<(), napi::Error>(())
  })?;

  BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL.with(|once_cell| {
    #[allow(clippy::unwrap_used)]
    let symbol = once_cell.get().unwrap();
    properties.push(
      Property::new()
        .with_name(env, symbol)?
        .with_getter_closure(|env, this| {
          let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
          let env_ref = &env;
          let result = wrapped_value.with_ref(|module| {
            module
              .build_info()
              .build_dependencies
              .iter()
              .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
              .collect::<napi::Result<Vec<JsString>>>()
          });
          unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
        })
        .with_property_attributes(PropertyAttributes::Configurable),
    );
    Ok::<(), napi::Error>(())
  })?;

  Ok(())
}

thread_local! {
  static BUILD_INFO_PROPERTIES_BUFFER: RefCell<Vec<Property>> = const { RefCell::new(Vec::new()) };
}

impl ToNapiValue for BuildInfo {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    mut val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      let env_wrapper = Env::from_raw(env);
      let module_reference = val.module_reference.clone();
      let known = KnownBuildInfo::new(module_reference);
      let napi_val = ToNapiValue::to_napi_value(env, known)?;
      let mut object = Object::from_raw(env, napi_val);

      BUILD_INFO_PROPERTIES_BUFFER.with(|ref_cell| {
        let mut properties = ref_cell.borrow_mut();
        properties.clear();
        create_known_private_properties(&env_wrapper, &mut properties)?;

        let commit_custom_fields_fn: napi::bindgen_prelude::Function<'_, (), ()> = env_wrapper
          .create_function_from_closure("commitCustomFieldsToRust", |ctx| {
            let object = ctx.this::<Object>()?;
            let env = ctx.env;
            let this: &mut KnownBuildInfo =
              FromNapiMutRef::from_napi_mut_ref(env.raw(), object.raw())?;

            this.with_mut(|module| {
              let mut extras = serde_json::Map::new();
              let names = Array::from_unknown(object.get_property_names()?.to_unknown())?;
              for index in 0..names.len() {
                if let Some(name) = names.get::<String>(index)?
                  && !KNOWN_BUILD_INFO_FIELD_NAMES.contains(name.as_str())
                {
                  let value = object.get_named_property::<Unknown>(&name)?;
                  if let Some(json_value) = unknown_to_json_value(value)? {
                    extras.insert(name, json_value);
                  }
                }
              }

              module.build_info_mut().extras = extras;

              Ok(())
            })
          })?;

        val.with_ref(|module| {
          let extras = &module.build_info().extras;
          properties.reserve(extras.len() + 1);
          for (key, value) in extras {
            let napi_val = ToNapiValue::to_napi_value(env, value)?;
            properties.push(
              Property::new()
                .with_utf8_name(key)?
                .with_value(&Object::from_raw(env, napi_val)),
            );
          }
          Ok(())
        })?;
        COMMIT_CUSTOM_FIELDS_SYMBOL.with(|once_cell| {
          #[allow(clippy::unwrap_used)]
          let symbol = once_cell.get().unwrap();
          properties.push(
            Property::new()
              .with_name(&env_wrapper, symbol)?
              .with_value(&commit_custom_fields_fn)
              .with_property_attributes(PropertyAttributes::Configurable),
          );
          Ok::<(), napi::Error>(())
        })?;
        object.define_properties(&properties)
      })?;

      Ok(napi_val)
    }
  }
}
