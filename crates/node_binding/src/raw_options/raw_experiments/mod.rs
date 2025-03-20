mod raw_cache;
mod raw_incremental;
mod raw_rspack_future;

use std::fmt;

use napi_derive::napi;
use raw_cache::{normalize_raw_experiment_cache_options, RawExperimentCacheOptions};
use raw_incremental::RawIncremental;
use raw_rspack_future::RawRspackFuture;
use rspack_core::{incremental::IncrementalPasses, Experiments};

use super::WithFalse;

#[napi(object, object_to_js = false)]
pub struct RawExperiments {
  pub layers: bool,
  pub top_level_await: bool,
  #[napi(ts_type = "false | { [key: string]: boolean }")]
  pub incremental: Option<WithFalse<RawIncremental>>,
  pub parallel_code_splitting: bool,
  pub rspack_future: Option<RawRspackFuture>,
  #[napi(
    ts_type = r#"boolean | { type: "persistent" } & RawExperimentCacheOptionsPersistent | { type: "memory" }"#
  )]
  pub cache: RawExperimentCacheOptions,
  #[napi(
    ts_type = r#"boolean | { http_client?: Function, allowedUris?: Array<string>, cacheLocation?: string, frozen?: boolean, lockfileLocation?: string, proxy?: string, upgrade?: boolean }"#,
    js_name = "buildHttp"
  )]
  pub build_http: Option<napi::JsUnknown>,
}

impl fmt::Debug for RawExperiments {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RawExperiments")
      .field("layers", &self.layers)
      .field("top_level_await", &self.top_level_await)
      .field("incremental", &self.incremental)
      .field("parallel_code_splitting", &self.parallel_code_splitting)
      .field("rspack_future", &self.rspack_future)
      .field("cache", &self.cache)
      .field("build_http", &"[JsUnknown]")
      .finish()
  }
}

impl From<RawExperiments> for Experiments {
  fn from(value: RawExperiments) -> Self {
    Self {
      incremental: match value.incremental {
        Some(value) => match value {
          WithFalse::True(value) => value.into(),
          WithFalse::False => IncrementalPasses::empty(),
        },
        None => IncrementalPasses::empty(),
      },
      parallel_code_splitting: value.parallel_code_splitting,
      layers: value.layers,
      top_level_await: value.top_level_await,
      rspack_future: value.rspack_future.unwrap_or_default().into(),
      cache: normalize_raw_experiment_cache_options(value.cache),
    }
  }
}
