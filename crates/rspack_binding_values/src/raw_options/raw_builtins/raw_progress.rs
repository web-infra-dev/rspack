use std::sync::Arc;

use derive_more::Debug;
use napi::Either;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_progress::{ProgressPluginDisplayOptions, ProgressPluginOptions};

type HandlerFn = ThreadsafeFunction<(f64, String, Vec<String>), ()>;
#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawProgressPluginOptions {
  // the prefix name of progress bar
  pub prefix: Option<String>,
  // tells ProgressPlugin to collect profile data for progress steps.
  pub profile: Option<bool>,
  // the template of progress bar
  pub template: Option<String>,
  // the tick string sequence for spinners, if it's string then it will be split into characters
  pub tick: Option<Either<String, Vec<String>>>,
  // the progress characters
  pub progress_chars: Option<String>,
  // the handler for progress event
  #[debug(skip)]
  #[napi(ts_type = "(percent: number, msg: string, items: string[]) => void")]
  pub handler: Option<HandlerFn>,
}

impl From<RawProgressPluginOptions> for ProgressPluginOptions {
  fn from(value: RawProgressPluginOptions) -> Self {
    if let Some(f) = value.handler {
      Self::Handler(Arc::new(move |percent, msg, items| {
        f.blocking_call_with_sync((percent, msg, items))
      }))
    } else {
      Self::Default(ProgressPluginDisplayOptions {
        prefix: value.prefix.unwrap_or_default(),
        profile: value.profile.unwrap_or_default(),
        template: value.template.unwrap_or(
          "● {prefix:.bold} {bar:25.green/white.dim} ({percent}%) {wide_msg:.dim}".to_string(),
        ),
        progress_chars: value.progress_chars.unwrap_or("━━".to_string()),
        tick_strings: value.tick.map(|tick| match tick {
          Either::A(str) => str.chars().map(|c| c.to_string()).collect(),
          Either::B(vec) => vec,
        }),
      })
    }
  }
}
