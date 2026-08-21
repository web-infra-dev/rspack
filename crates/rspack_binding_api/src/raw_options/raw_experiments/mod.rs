mod raw_incremental;
mod raw_new_cache;

use napi_derive::napi;
pub use raw_incremental::RawIncremental;
pub use raw_new_cache::RawNewCache;
use rspack_core::{Experiments, NewCacheOptions, runtime_mode::RuntimeMode};
use rspack_regex::RspackRegex;

use super::WithFalse;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawExperiments {
  #[napi(ts_type = "false | Array<RegExp>")]
  pub use_input_file_system: Option<WithFalse<Vec<RspackRegex>>>,
  pub css: Option<bool>,
  #[napi(ts_type = "false | RawNewCache")]
  pub new_cache: WithFalse<RawNewCache>,
  pub defer_import: bool,
  pub source_import: bool,
  pub faster_module_concatenation: bool,
  pub pure_functions: bool,
  #[napi(ts_type = "\"webpack\" | \"rspack\"")]
  pub runtime_mode: Option<String>,
}

impl From<RawExperiments> for Experiments {
  fn from(value: RawExperiments) -> Self {
    let runtime_mode = if value.runtime_mode.as_deref() == Some("rspack") {
      RuntimeMode::Rspack
    } else {
      RuntimeMode::Webpack
    };

    Self {
      css: value.css.unwrap_or(false),
      new_cache: match value.new_cache {
        WithFalse::False => NewCacheOptions::default(),
        WithFalse::True(value) => value.into(),
      },
      defer_import: value.defer_import,
      source_import: value.source_import,
      faster_module_concatenation: value.faster_module_concatenation,
      pure_functions: value.pure_functions,
      runtime_mode,
    }
  }
}
