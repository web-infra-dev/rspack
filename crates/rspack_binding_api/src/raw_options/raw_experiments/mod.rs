mod raw_incremental;

use napi_derive::napi;
use raw_incremental::RawIncremental;
use rspack_core::{Experiments, incremental::IncrementalOptions};
use rspack_regex::RspackRegex;

use super::WithFalse;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawExperiments {
  pub top_level_await: bool,
  #[napi(ts_type = "false | { [key: string]: boolean }")]
  pub incremental: Option<WithFalse<RawIncremental>>,
  #[napi(ts_type = "false | Array<RegExp>")]
  pub use_input_file_system: Option<WithFalse<Vec<RspackRegex>>>,
  pub css: Option<bool>,
  pub lazy_barrel: bool,
  pub defer_import: bool,
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
      top_level_await: value.top_level_await,
      css: value.css.unwrap_or(false),
      lazy_barrel: value.lazy_barrel,
      defer_import: value.defer_import,
    }
  }
}
