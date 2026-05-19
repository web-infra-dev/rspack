mod raw_snapshot;
mod raw_storage;

use core::panic;

use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, JsObjectValue, Object, TypeName, ValidateNapiValue},
};
use napi_derive::napi;
use raw_snapshot::RawSnapshotOptions;
use raw_storage::RawStorageOptions;
use rspack_core::{CacheOptions, cache::persistent::PersistentCacheOptions};

#[derive(Debug)]
#[napi(object)]
pub struct RawCacheOptionsPersistent {
  pub build_dependencies: Option<Vec<String>>,
  pub version: Option<String>,
  pub snapshot: Option<RawSnapshotOptions>,
  pub storage: Option<RawStorageOptions>,
  pub portable: Option<bool>,
  pub readonly: Option<bool>,
}

impl From<RawCacheOptionsPersistent> for PersistentCacheOptions {
  fn from(value: RawCacheOptionsPersistent) -> Self {
    Self {
      build_dependencies: value
        .build_dependencies
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect(),
      version: value.version.unwrap_or_default(),
      snapshot: value.snapshot.unwrap_or_default().into(),
      storage: value.storage.unwrap_or_default().into(),
      portable: value.portable.unwrap_or_default(),
      readonly: value.readonly.unwrap_or_default(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawCacheOptionsMemory {
  pub max_generations: Option<u32>,
}

#[derive(Debug)]
pub enum InnerCacheOptions {
  Memory(RawCacheOptionsMemory),
  Persistent(RawCacheOptionsPersistent),
}

impl TypeName for InnerCacheOptions {
  fn type_name() -> &'static str {
    "InnerCacheOptions"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for InnerCacheOptions {}

impl FromNapiValue for InnerCacheOptions {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe {
      let o = Object::from_napi_value(env, napi_val)?;
      let t = o.get_named_property::<String>("type")?;

      let v = match &*t {
        "persistent" => {
          let o = RawCacheOptionsPersistent::from_napi_value(env, napi_val)?;
          Self::Persistent(o)
        }
        "memory" => {
          let o = RawCacheOptionsMemory::from_napi_value(env, napi_val)?;
          Self::Memory(o)
        }
        _ => panic!("Unexpected cache type: {t}, expected 'persistent' or 'memory'"),
      };

      Ok(v)
    }
  }
}

pub type RawCacheOptions = Either<bool, InnerCacheOptions>;

pub fn normalize_raw_cache(options: RawCacheOptions) -> CacheOptions {
  match options {
    Either::A(options) => {
      if options {
        CacheOptions::Memory { max_generations: 1 }
      } else {
        CacheOptions::Disabled
      }
    }
    Either::B(options) => match options {
      InnerCacheOptions::Persistent(options) => CacheOptions::Persistent(options.into()),
      InnerCacheOptions::Memory(options) => CacheOptions::Memory {
        max_generations: options.max_generations.unwrap_or(1),
      },
    },
  }
}
