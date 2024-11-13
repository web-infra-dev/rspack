mod raw_snapshot;
mod raw_storage;

use napi::Either;
use napi_derive::napi;
use raw_snapshot::RawExperimentSnapshotOptions;
use raw_storage::RawStorageOption;
use rspack_core::{cache::persistent::PersistentCacheOptions, ExperimentCacheOptions};

pub type RawExperimentCacheOptions =
  Either<RawExperimentCacheOptionsPersistent, RawExperimentCacheOptionsCommon>;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperimentCacheOptionsCommon {
  #[napi(ts_type = r#""disable"|"memory""#)]
  pub r#type: String,
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperimentCacheOptionsPersistent {
  #[napi(ts_type = r#""persistent""#)]
  pub r#type: String,
  //  pub build_dependencies: Vec<String>,
  //  pub version: String,
  pub snapshot: RawExperimentSnapshotOptions,
  pub storage: Vec<RawStorageOption>,
}

pub fn normalize_raw_experiment_cache_options(
  options: RawExperimentCacheOptions,
) -> ExperimentCacheOptions {
  match options {
    Either::A(persistent_options) => ExperimentCacheOptions::Persistent(PersistentCacheOptions {
      snapshot: persistent_options.snapshot.into(),
      storage: persistent_options
        .storage
        .into_iter()
        .map(Into::into)
        .collect(),
    }),
    Either::B(options) => match options.r#type.as_str() {
      "disable" => ExperimentCacheOptions::Disabled,
      "memory" => ExperimentCacheOptions::Memory,
      _ => panic!("unsupported cache type"),
    },
  }
}
