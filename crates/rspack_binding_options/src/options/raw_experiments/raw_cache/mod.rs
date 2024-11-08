mod raw_snapshot;
mod raw_storage;

use napi_derive::napi;
use raw_snapshot::RawSnapshotOptions;
use raw_storage::RawStorageOption;
use rspack_core::{cache::persistent::PersistentCacheOptions, ExperimentCacheOptions};

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperimentCacheOptions {
  #[napi(ts_type = r#""disable"|"memory"|"persistent""#)]
  pub r#type: String,
  //  pub build_dependencies: Vec<String>,
  //  pub version: String,
  pub snapshot: RawSnapshotOptions,
  pub storage: Vec<RawStorageOption>,
}

impl From<RawExperimentCacheOptions> for ExperimentCacheOptions {
  fn from(value: RawExperimentCacheOptions) -> Self {
    match value.r#type.as_str() {
      "disable" => ExperimentCacheOptions::Disabled,
      "memory" => ExperimentCacheOptions::Memory,
      "persistent" => ExperimentCacheOptions::Persistent(PersistentCacheOptions {
        snapshot: value.snapshot.into(),
        storage: value.storage.into_iter().map(Into::into).collect(),
      }),
      _ => panic!("unsupported cache type"),
    }
  }
}
