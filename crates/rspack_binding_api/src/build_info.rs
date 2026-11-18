use std::{
  cell::RefCell,
  rc::{Rc, Weak},
  sync::LazyLock,
};

use napi::{
  Env, JsString, JsValue, Property, PropertyAttributes, Unknown,
  bindgen_prelude::{
    Array, FromNapiMutRef, FromNapiValue, JsObjectValue, Object, ToNapiValue, ValidateNapiValue,
  },
};
use rspack_core::WeakBindingCell;
use rspack_napi::{OneShotRef, unknown_to_json_value};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{module::ModuleHandle, shared_properties::define_shared_properties, symbols};

type AssetMap = FxHashMap<String, rspack_core::CompilationAsset>;

enum AssetsOwner {
  Map(WeakBindingCell<AssetMap>),
  Module(Weak<ModuleHandle>),
}

// Record<string, Source>
#[napi]
pub struct Assets {
  owner: AssetsOwner,
}

impl Assets {
  pub fn new(map: WeakBindingCell<AssetMap>) -> Self {
    Self {
      owner: AssetsOwner::Map(map),
    }
  }

  fn with_ref<T>(&self, f: impl FnOnce(&AssetMap) -> napi::Result<T>) -> napi::Result<T> {
    match &self.owner {
      AssetsOwner::Map(map) => {
        let map = map.upgrade().ok_or_else(|| {
          napi::Error::from_reason("Unable to access assets. The assets has been dropped by Rust.")
        })?;
        f(map.as_ref())
      }
      AssetsOwner::Module(module) => {
        let module = module.upgrade().ok_or_else(|| {
          napi::Error::from_reason("Unable to access assets after their Module has been released")
        })?;
        module.with_local_ref(|module| f(&module.build_info().assets))
      }
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
  module: Weak<ModuleHandle>,
  worker_assets: Option<OneShotRef>,
}

impl KnownBuildInfo {
  fn new(module: &Rc<ModuleHandle>) -> Self {
    Self {
      module: Rc::downgrade(module),
      worker_assets: None,
    }
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object<'_>> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(Object::from_raw(raw_env, napi_val))
  }

  fn module(&self) -> napi::Result<Rc<ModuleHandle>> {
    self.module.upgrade().ok_or_else(|| {
      napi::Error::from_reason("Unable to access BuildInfo after its Module has been released")
    })
  }

  fn with_ref<T>(
    &self,
    f: impl FnOnce(&dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    self.module()?.with_local_ref(f)
  }

  fn with_mut<T>(
    &self,
    f: impl FnOnce(&mut dyn rspack_core::Module) -> napi::Result<T>,
  ) -> napi::Result<T> {
    self.module()?.with_mut(f)
  }

  fn assets(&mut self, env: &Env) -> napi::Result<Object<'_>> {
    let handle = self.module()?;
    let raw_env = env.raw();
    let value = if handle.worker {
      handle.with_local_ref(|_| Ok(()))?;
      // Keep the asset view in this worker's JS environment. Its reads still
      // resolve through the module handle and observe scope revocation.
      let assets = match &self.worker_assets {
        Some(assets) => assets,
        None => self.worker_assets.insert(OneShotRef::new(
          raw_env,
          Assets {
            owner: AssetsOwner::Module(Rc::downgrade(&handle)),
          },
        )?),
      };
      unsafe { ToNapiValue::to_napi_value(raw_env, assets)? }
    } else {
      let assets = handle.with_local_ref(|module| Ok(module.build_info().assets.reflector()))?;
      unsafe { ToNapiValue::to_napi_value(raw_env, assets)? }
    };
    Ok(Object::from_raw(raw_env, value))
  }
}

// KnownBuildInfo & Record<string, any>
pub struct BuildInfo<'a>(&'a Rc<ModuleHandle>);

impl<'a> BuildInfo<'a> {
  pub(crate) fn new(module: &'a Rc<ModuleHandle>) -> Self {
    Self(module)
  }

  pub fn get_jsobject(self, env: &Env) -> napi::Result<Object<'_>> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(Object::from_raw(raw_env, napi_val))
  }
}

fn create_known_private_properties(env: &Env, properties: &mut Vec<Property>) -> napi::Result<()> {
  properties.push(
    Property::new()
      .with_name(env, env.symbol_for(symbols::BUILD_INFO_ASSETS_SYMBOL)?)?
      .with_getter_closure(|env, this| {
        // Native references do not escape this synchronous getter.
        let _borrow_scope = unsafe { napi::bindgen_prelude::NativeBorrowScope::new() };
        let wrapped_value = unsafe { KnownBuildInfo::from_napi_mut_ref(env.raw(), this.raw())? };
        wrapped_value.assets(&env)
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

thread_local! {
  static BUILD_INFO_PROPERTIES_BUFFER: RefCell<Vec<Property>> = const { RefCell::new(Vec::new()) };
}

impl ToNapiValue for BuildInfo<'_> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      let env_wrapper = Env::from_raw(env);
      let known = KnownBuildInfo::new(val.0);
      let napi_val = ToNapiValue::to_napi_value(env, known)?;
      let mut object = Object::from_raw(env, napi_val);

      BUILD_INFO_PROPERTIES_BUFFER.with(|ref_cell| {
        let mut properties = ref_cell.borrow_mut();
        properties.clear();
        define_shared_properties::<KnownBuildInfo>(&env_wrapper, object, || {
          let mut properties = Vec::with_capacity(6);
          create_known_private_properties(&env_wrapper, &mut properties)?;

          let commit_custom_fields_fn: napi::bindgen_prelude::Function<'_, (), ()> = env_wrapper
            .create_function_from_closure("commitCustomFieldsToRust", |ctx| {
              let object = ctx.this::<Object>()?;
              let env = ctx.env;
              <&KnownBuildInfo>::validate(env.raw(), object.raw())?;
              // Reading custom properties can invoke user getters that re-enter
              // BuildInfo or the module cache. Finish that work before borrowing either.
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

              let _borrow_scope = napi::bindgen_prelude::NativeBorrowScope::new();
              let this: &mut KnownBuildInfo =
                FromNapiMutRef::from_napi_mut_ref(env.raw(), object.raw())?;
              this.with_mut(|module| {
                module.build_info_mut().extras = extras;
                Ok(())
              })
            })?;

          properties.push(
            Property::new()
              .with_name(
                &env_wrapper,
                env_wrapper.symbol_for(symbols::COMMIT_CUSTOM_FIELDS_SYMBOL)?,
              )?
              .with_value(&commit_custom_fields_fn)
              .with_property_attributes(PropertyAttributes::Configurable),
          );
          Ok(properties)
        })?;

        val.0.with_local_ref(|module| {
          let extras = &module.build_info().extras;
          properties.reserve(extras.len());
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
        object.define_properties(&properties)
      })?;

      Ok(napi_val)
    }
  }
}
