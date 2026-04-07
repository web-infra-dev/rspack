mod raw_incremental;

use napi_derive::napi;
pub use raw_incremental::RawIncremental;
use rspack_core::Experiments;

use super::WithFalse;
use crate::js_regex::JsRegExp;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawExperiments {
  #[napi(ts_type = "false | Array<RegExp>")]
  pub use_input_file_system: Option<WithFalse<Vec<JsRegExp>>>,
  pub css: Option<bool>,
  pub defer_import: bool,
  pub pure_functions: bool,
}

impl From<RawExperiments> for Experiments {
  fn from(value: RawExperiments) -> Self {
    Self {
      css: value.css.unwrap_or(false),
      defer_import: value.defer_import,
      pure_functions: value.pure_functions,
    }
  }
}
