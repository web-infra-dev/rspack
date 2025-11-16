mod raw_cache;
mod raw_incremental;
mod raw_rspack_future;

use napi_derive::napi;
use raw_cache::{RawExperimentCacheOptions, normalize_raw_experiment_cache_options};
use raw_incremental::RawIncremental;
use raw_rspack_future::RawRspackFuture;
use rspack_core::{Experiments, incremental::IncrementalOptions};
use rspack_regex::RspackRegex;

use super::WithFalse;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawExperiments {
  pub top_level_await: bool,
  #[napi(ts_type = "false | { [key: string]: boolean }")]
  pub incremental: Option<WithFalse<RawIncremental>>,
  pub parallel_code_splitting: bool,
  pub rspack_future: Option<RawRspackFuture>,
  #[napi(
    ts_type = r#"boolean | { type: "persistent" } & RawExperimentCacheOptionsPersistent | { type: "memory" }"#
  )]
  pub cache: RawExperimentCacheOptions,
  #[napi(ts_type = "false | Array<RegExp>")]
  pub use_input_file_system: Option<WithFalse<Vec<RspackRegex>>>,
  pub css: Option<bool>,
  pub inline_const: bool,
  pub inline_enum: bool,
  pub type_reexports_presence: bool,
  pub lazy_barrel: bool,
  pub defer_import: bool,
  pub mf_async_startup: bool,
}

impl From<RawExperiments> for Experiments {
  fn from(value: RawExperiments) -> Self {
    Self {
      incremental: match value.incremental {
        Some(value) => match value {
          WithFalse::True(value) => value.into(),
          WithFalse::False => IncrementalOptions::empty_passes(),
        },
        None => IncrementalOptions::empty_passes(),
      },
      parallel_code_splitting: value.parallel_code_splitting,
      top_level_await: value.top_level_await,
      rspack_future: value.rspack_future.unwrap_or_default().into(),
      cache: normalize_raw_experiment_cache_options(value.cache),
      css: value.css.unwrap_or(false),
      inline_const: value.inline_const,
      inline_enum: value.inline_enum,
      type_reexports_presence: value.type_reexports_presence,
      lazy_barrel: value.lazy_barrel,
      defer_import: value.defer_import,
      mf_async_startup: value.mf_async_startup,
    }
  }
}
