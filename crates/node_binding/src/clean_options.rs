use std::{str::FromStr, sync::Arc};

use ::napi::bindgen_prelude::Either3;
use napi_derive::napi;
use rspack_core::CleanOptions;
use rspack_napi::{napi, threadsafe_function::ThreadsafeFunction};
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;

/// File clean options
///
/// This matches with:
/// - keep:
///   - If a string, keep the files under this path
#[napi(object, object_to_js = false)]
#[derive(Debug)]
pub struct JsCleanOptions {
  #[napi(ts_type = "string | RegExp | ((path: string) => boolean)")]
  pub keep: Option<Either3<String, RspackRegex, ThreadsafeFunction<String, bool>>>,
}

impl From<JsCleanOptions> for CleanOptions {
  fn from(value: JsCleanOptions) -> Self {
    match value.keep {
      Some(Either3::A(path)) => {
        CleanOptions::KeepPath(Utf8PathBuf::from_str(&path).expect("should be a valid path"))
      }
      Some(Either3::B(reg_exp)) => CleanOptions::KeepRegex(reg_exp),
      Some(Either3::C(func)) => CleanOptions::KeepFunc(Arc::new(move |path| {
        let func = func.clone();
        Box::pin(async move { func.call_with_sync(path).await })
      })),
      None => CleanOptions::CleanAll(false),
    }
  }
}
