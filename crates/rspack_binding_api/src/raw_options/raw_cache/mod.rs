mod raw_storage;

use std::time::Duration;

use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, JsObjectValue, Object, TypeName, ValidateNapiValue},
};
use napi_derive::napi;
use raw_storage::RawStorageOptions;
use rspack_core::{CacheOptions, FileSystemCacheOptions, PersistentCacheOptions};

#[derive(Debug)]
#[napi(object)]
pub struct RawCacheOptionsPersistent {
  pub build_dependencies: Option<Vec<String>>,
  pub version: Option<String>,
  pub max_age: u32,
  pub max_memory_generations: Option<u32>,
  pub storage: Option<RawStorageOptions>,
  pub portable: Option<bool>,
  pub readonly: Option<bool>,
}

impl TryFrom<RawCacheOptionsPersistent> for PersistentCacheOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawCacheOptionsPersistent) -> rspack_error::Result<Self> {
    let storage = value.storage.unwrap_or_default().normalize()?;
    Ok(Self {
      build_dependencies: value
        .build_dependencies
        .unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect(),
      version: value.version.unwrap_or_default(),
      storage,
      portable: value.portable.unwrap_or_default(),
      readonly: value.readonly.unwrap_or_default(),
      max_age: value.max_age.into(),
      max_memory_generations: value.max_memory_generations.into(),
    })
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawCacheOptionsMemory {
  pub max_generations: Option<u32>,
}

#[derive(Debug)]
#[napi(object)]
pub struct RawFileSystemCacheOptions {
  pub build_dependencies: Vec<String>,
  pub cache_directory: String,
  pub cache_location: String,
  pub version: String,
  pub readonly: bool,
  pub max_memory_generations: Option<u32>,
  pub idle_timeout: f64,
  pub idle_timeout_for_initial_store: f64,
  pub idle_timeout_after_large_changes: f64,
}

impl From<RawFileSystemCacheOptions> for FileSystemCacheOptions {
  fn from(value: RawFileSystemCacheOptions) -> Self {
    Self {
      build_dependencies: value
        .build_dependencies
        .into_iter()
        .map(Into::into)
        .collect(),
      cache_directory: value.cache_directory.into(),
      cache_location: value.cache_location.into(),
      version: value.version,
      readonly: value.readonly,
      max_memory_generations: value.max_memory_generations.into(),
      idle_timeout: Duration::from_millis(value.idle_timeout as u64),
      idle_timeout_for_initial_store: Duration::from_millis(
        value.idle_timeout_for_initial_store as u64,
      ),
      idle_timeout_after_large_changes: Duration::from_millis(
        value.idle_timeout_after_large_changes as u64,
      ),
    }
  }
}

#[derive(Debug)]
pub enum InnerCacheOptions {
  Memory(RawCacheOptionsMemory),
  Persistent(RawCacheOptionsPersistent),
  FileSystem(RawFileSystemCacheOptions),
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
        "filesystem" => {
          Self::FileSystem(RawFileSystemCacheOptions::from_napi_value(env, napi_val)?)
        }
        "persistent" => {
          let o = RawCacheOptionsPersistent::from_napi_value(env, napi_val)?;
          Self::Persistent(o)
        }
        "memory" => {
          let o = RawCacheOptionsMemory::from_napi_value(env, napi_val)?;
          Self::Memory(o)
        }
        _ => {
          return Err(napi::Error::from_reason(format!(
            "Unexpected cache type: {t}, expected 'persistent', 'filesystem' or 'memory'"
          )));
        }
      };

      Ok(v)
    }
  }
}

pub type RawCacheOptions = Either<bool, InnerCacheOptions>;

pub fn normalize_raw_cache(options: RawCacheOptions) -> rspack_error::Result<CacheOptions> {
  Ok(match options {
    Either::A(options) => {
      if options {
        CacheOptions::Memory { max_generations: 1 }
      } else {
        CacheOptions::Disabled
      }
    }
    Either::B(options) => match options {
      InnerCacheOptions::Persistent(options) => CacheOptions::Persistent(options.try_into()?),
      InnerCacheOptions::FileSystem(options) => CacheOptions::FileSystem(options.into()),
      InnerCacheOptions::Memory(options) => CacheOptions::Memory {
        max_generations: options.max_generations.unwrap_or(1),
      },
    },
  })
}
