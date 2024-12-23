mod raw_snapshot;
mod raw_storage;

use core::panic;

use napi::bindgen_prelude::Either3;
use napi_derive::napi;
use raw_snapshot::RawExperimentSnapshotOptions;
use raw_storage::RawStorageOptions;
use rspack_core::{cache::persistent::PersistentCacheOptions, ExperimentCacheOptions};

pub type RawExperimentCacheOptions =
  Either3<bool, RawExperimentCacheOptionsMemory, RawExperimentCacheOptionsPersistent>;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperimentCacheOptionsPersistent {
  #[napi(ts_type = r#""persistent""#)]
  pub r#type: String,
  pub build_dependencies: Vec<String>,
  pub version: String,
  pub snapshot: RawExperimentSnapshotOptions,
  pub storage: RawStorageOptions,
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperimentCacheOptionsMemory {
  #[napi(ts_type = r#""memory" | "disable""#)]
  pub r#type: String,
}

pub fn normalize_raw_experiment_cache_options(
  options: RawExperimentCacheOptions,
) -> ExperimentCacheOptions {
  match options {
    Either3::C(persistent_options) if persistent_options.r#type == "persistent" => {
      ExperimentCacheOptions::Persistent(PersistentCacheOptions {
        build_dependencies: persistent_options
          .build_dependencies
          .into_iter()
          .map(Into::into)
          .collect(),
        version: persistent_options.version,
        snapshot: persistent_options.snapshot.into(),
        storage: persistent_options.storage.into(),
      })
    }
    Either3::B(options) if options.r#type == "memory" => ExperimentCacheOptions::Memory,
    Either3::B(options) if options.r#type == "disable" => ExperimentCacheOptions::Disabled,
    Either3::A(options) => {
      if options {
        ExperimentCacheOptions::Memory
      } else {
        ExperimentCacheOptions::Disabled
      }
    }
    _ => panic!("Invalid cache options"),
  }
}
