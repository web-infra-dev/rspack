mod raw_snapshot;
mod raw_storage;

use core::panic;

use napi::{
  bindgen_prelude::{FromNapiValue, Object, TypeName, ValidateNapiValue},
  Either,
};
use napi_derive::napi;
use raw_snapshot::RawExperimentSnapshotOptions;
use raw_storage::RawStorageOptions;
use rspack_core::{cache::persistent::PersistentCacheOptions, ExperimentCacheOptions};

pub type RawExperimentCacheOptions = Either<bool, RawExperimentCache>;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperimentCacheOptionsPersistent {
  pub build_dependencies: Option<Vec<String>>,
  pub version: Option<String>,
  pub snapshot: Option<RawExperimentSnapshotOptions>,
  pub storage: Option<RawStorageOptions>,
}

impl From<RawExperimentCacheOptionsPersistent> for PersistentCacheOptions {
  fn from(value: RawExperimentCacheOptionsPersistent) -> Self {
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
    }
  }
}

#[derive(Debug, Default)]
pub enum RawExperimentCache {
  #[default]
  Memory,
  Persistent(RawExperimentCacheOptionsPersistent),
}

impl TypeName for RawExperimentCache {
  fn type_name() -> &'static str {
    "RawExperimentCache"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for RawExperimentCache {}

impl FromNapiValue for RawExperimentCache {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let o = Object::from_napi_value(env, napi_val)?;
    let t = o.get_named_property::<String>("type")?;

    let v = match &*t {
      "persistent" => {
        let o = RawExperimentCacheOptionsPersistent::from_napi_value(env, napi_val)?;
        Self::Persistent(o)
      }
      "memory" => Self::Memory,
      _ => panic!("Unexpected cache type: {t}, expected 'persistent' or 'memory'"),
    };

    Ok(v)
  }
}

pub fn normalize_raw_experiment_cache_options(
  options: RawExperimentCacheOptions,
) -> ExperimentCacheOptions {
  match options {
    Either::A(options) => {
      if options {
        ExperimentCacheOptions::Memory
      } else {
        ExperimentCacheOptions::Disabled
      }
    }
    Either::B(options) => match options {
      RawExperimentCache::Persistent(options) => ExperimentCacheOptions::Persistent(options.into()),
      RawExperimentCache::Memory => ExperimentCacheOptions::Memory,
    },
  }
}
