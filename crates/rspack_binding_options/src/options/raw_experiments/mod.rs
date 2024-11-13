mod raw_cache;
mod raw_incremental;
mod raw_rspack_future;

use napi_derive::napi;
use raw_cache::{normalize_raw_experiment_cache_options, RawExperimentCacheOptions};
use raw_incremental::RawIncremental;
use raw_rspack_future::RawRspackFuture;
use rspack_core::{incremental::IncrementalPasses, Experiments};

#[derive(Debug)]
#[napi(object)]
pub struct RawExperiments {
  pub layers: bool,
  pub top_level_await: bool,
  pub incremental: Option<RawIncremental>,
  pub rspack_future: RawRspackFuture,
  #[napi(ts_type = r#"RawExperimentCacheOptionsPersistent | RawExperimentCacheOptionsCommon"#)]
  pub cache: RawExperimentCacheOptions,
}

impl From<RawExperiments> for Experiments {
  fn from(value: RawExperiments) -> Self {
    Self {
      incremental: match value.incremental {
        Some(value) => value.into(),
        None => IncrementalPasses::empty(),
      },
      layers: value.layers,
      top_level_await: value.top_level_await,
      rspack_future: value.rspack_future.into(),
      cache: normalize_raw_experiment_cache_options(value.cache),
    }
  }
}
