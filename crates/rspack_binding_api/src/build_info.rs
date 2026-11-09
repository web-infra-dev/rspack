use std::sync::LazyLock;

use napi::{
  Env, JsString, JsValue, Property, PropertyAttributes, Unknown,
  bindgen_prelude::{
    Array, FromNapiMutRef, FromNapiValue, JsObjectValue, Object, ToNapiValue, ValidateNapiValue,
  },
};
use rspack_core::{CompilerId, ModuleIdentifier, WeakBindingCell};
use rspack_napi::unknown_to_json_value;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  module::{ModuleHandle, with_module_instance},
  shared_properties::define_shared_properties,
  symbols,
};

pub(crate) type BuildInfoExtras = serde_json::Map<String, serde_json::Value>;

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

enum BuildInfoTarget {
  Compilation(CompilerId, ModuleIdentifier),
  Worker(Box<ModuleHandle>),
}

#[napi]
pub struct KnownBuildInfo {
  target: BuildInfoTarget,
}

impl KnownBuildInfo {
  pub(crate) fn new(module: &ModuleHandle) -> Self {
    let target = if module.worker_scope.is_some() {
      BuildInfoTarget::Worker(Box::new(ModuleHandle::new(&module.to_module_object())))
    } else {
      BuildInfoTarget::Compilation(module.compiler_id, module.identifier)
    };
    Self { target }
  }

  fn with_handle<T>(&self, f: impl FnOnce(&ModuleHandle) -> napi::Result<T>) -> napi::Result<T> {
    match &self.target {
      BuildInfoTarget::Compilation(compiler_id, module_id) => {
        with_module_instance(*compiler_id, *module_id, f)
      }
      BuildInfoTarget::Worker(module) => f(module),
    }
  }

  fn with_ref<T>(
    &self,
    f: impl FnOnce(&dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    self.with_handle(|module| module.with_local_ref(f))
  }
}

/// Synchronize changed native fields while preserving JavaScript-only fields.
pub(crate) fn sync_build_info_extras(
  env: &Env,
  mut object: Object<'_>,
  previous: &BuildInfoExtras,
  current: &BuildInfoExtras,
) -> napi::Result<()> {
  for key in previous.keys() {
    if !current.contains_key(key) {
      object.delete_named_property(key)?;
    }
  }
  let mut properties = Vec::new();
  for (key, value) in current {
    if previous.get(key) != Some(value) {
      let value =
        unsafe { Object::from_raw(env.raw(), ToNapiValue::to_napi_value(env.raw(), value)?) };
      properties.push(Property::new().with_utf8_name(key)?.with_value(&value));
    }
  }
  if !properties.is_empty() {
    object.define_properties(&properties)?;
  }
  Ok(())
}

fn create_known_private_properties(env: &Env, properties: &mut Vec<Property>) -> napi::Result<()> {
  properties.push(
    Property::new()
      .with_name(env, env.symbol_for(symbols::BUILD_INFO_ASSETS_SYMBOL)?)?
      .with_getter_closure(|env, this| {
        // Native references do not escape this synchronous getter.
        let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
        let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
        if matches!(wrapped_value.target, BuildInfoTarget::Worker(_)) {
          return Err(napi::Error::from_reason(
            "Reading Module buildInfo.assets is unavailable with `parallel: 'pure'`.",
          ));
        }
        wrapped_value.with_ref(|module| Ok(module.build_info().assets.reflector()))
      })
      .with_property_attributes(PropertyAttributes::Configurable),
  );

  properties.push(
    Property::new()
      .with_name(
        env,
        env.symbol_for(symbols::BUILD_INFO_FILE_DEPENDENCIES_SYMBOL)?,
      )?
      .with_getter_closure(|env, this| {
        // Native references do not escape this synchronous getter.
        let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
        let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
        let env_ref = &env;
        let result = wrapped_value.with_ref(|module| {
          module
            .build_info()
            .dependencies
            .file
            .iter()
            .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
            .collect::<napi::Result<Vec<JsString>>>()
        });
        unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
      })
      .with_property_attributes(PropertyAttributes::Configurable),
  );

  properties.push(
    Property::new()
      .with_name(
        env,
        env.symbol_for(symbols::BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL)?,
      )?
      .with_getter_closure(|env, this| {
        // Native references do not escape this synchronous getter.
        let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
        let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
        let env_ref = &env;
        let result = wrapped_value.with_ref(|module| {
          module
            .build_info()
            .dependencies
            .context
            .iter()
            .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
            .collect::<napi::Result<Vec<JsString>>>()
        });
        unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
      })
      .with_property_attributes(PropertyAttributes::Configurable),
  );

  properties.push(
    Property::new()
      .with_name(
        env,
        env.symbol_for(symbols::BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL)?,
      )?
      .with_getter_closure(|env, this| {
        // Native references do not escape this synchronous getter.
        let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
        let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
        let env_ref = &env;
        let result = wrapped_value.with_ref(|module| {
          module
            .build_info()
            .dependencies
            .missing
            .iter()
            .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
            .collect::<napi::Result<Vec<JsString>>>()
        });
        unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
      })
      .with_property_attributes(PropertyAttributes::Configurable),
  );

  properties.push(
    Property::new()
      .with_name(
        env,
        env.symbol_for(symbols::BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL)?,
      )?
      .with_getter_closure(|env, this| {
        // Native references do not escape this synchronous getter.
        let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
        let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
        let env_ref = &env;
        let result = wrapped_value.with_ref(|module| {
          module
            .build_info()
            .dependencies
            .build
            .iter()
            .map(|dependency| env_ref.create_string(dependency.to_string_lossy().as_ref()))
            .collect::<napi::Result<Vec<JsString>>>()
        });
        unsafe { ToNapiValue::to_napi_value(env.raw(), result) }
      })
      .with_property_attributes(PropertyAttributes::Configurable),
  );

  Ok(())
}

impl KnownBuildInfo {
  pub(crate) fn get_jsobject(self, env: &Env) -> napi::Result<(Object<'_>, BuildInfoExtras)> {
    let extras = self.with_ref(|module| Ok(module.build_info().extras.clone()))?;
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    let object = Object::from_raw(raw_env, napi_val);

    define_shared_properties::<KnownBuildInfo>(env, object, || {
      let mut properties = Vec::with_capacity(6);
      create_known_private_properties(env, &mut properties)?;

      let commit_custom_fields_fn: napi::bindgen_prelude::Function<'_, (), ()> = env
        .create_function_from_closure("commitCustomFieldsToRust", |ctx| {
          let object = ctx.this::<Object>()?;
          let env = ctx.env;
          unsafe { <&KnownBuildInfo>::validate(env.raw(), object.raw())? };
          // Reading custom properties can invoke JS. Convert them before
          // borrowing the native module or the instance cache.
          let mut extras = BuildInfoExtras::new();
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

          let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
          let this: &mut KnownBuildInfo =
            unsafe { FromNapiMutRef::from_napi_mut_ref(env.raw(), object.raw())? };
          this.with_handle(|handle| {
            let build_info = handle.as_mut()?.build_info_mut();
            handle.record_native_build_info_extras(&extras);
            build_info.extras = extras;
            Ok(())
          })
        })?;

      properties.push(
        Property::new()
          .with_name(env, env.symbol_for(symbols::COMMIT_CUSTOM_FIELDS_SYMBOL)?)?
          .with_value(&commit_custom_fields_fn)
          .with_property_attributes(PropertyAttributes::Configurable),
      );
      Ok(properties)
    })?;

    sync_build_info_extras(env, object, &BuildInfoExtras::new(), &extras)?;
    Ok((object, extras))
  }
}
