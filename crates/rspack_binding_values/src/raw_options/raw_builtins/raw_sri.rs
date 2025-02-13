use std::sync::Arc;

use derive_more::Debug;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_sri::{IntegrityCallbackData, SRIHashFunction, SRIPluginOptions};

#[derive(Debug)]
#[napi(object, object_to_js = false, js_name = "RawSRIPluginOptions")]
pub struct RawSRIPluginOptions {
  #[debug(skip)]
  #[napi(ts_type = "(data: RawIntegrityData) => void")]
  pub integrity_callback: Option<ThreadsafeFunction<RawIntegrityData, ()>>,
  pub hash_func_names: Vec<String>,
  #[napi(ts_type = "\"JavaScript\" | \"Native\" | \"Disabled\"")]
  pub html_plugin: String,
}

impl From<RawSRIPluginOptions> for SRIPluginOptions {
  fn from(options: RawSRIPluginOptions) -> Self {
    Self {
      integrity_callback: if let Some(func) = options.integrity_callback {
        Some(Arc::new(move |data| {
          func.blocking_call_with_sync(data.into())
        }))
      } else {
        None
      },
      hash_func_names: options
        .hash_func_names
        .into_iter()
        .map(SRIHashFunction::from)
        .collect::<Vec<_>>(),
      html_plugin: options.html_plugin.into(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawIntegrityData {
  pub integerities: Vec<RawIntegrityItem>,
}

impl From<IntegrityCallbackData> for RawIntegrityData {
  fn from(data: IntegrityCallbackData) -> Self {
    Self {
      integerities: data
        .integerities
        .into_iter()
        .map(|(asset, integrity)| RawIntegrityItem { asset, integrity })
        .collect::<Vec<_>>(),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawIntegrityItem {
  pub asset: String,
  pub integrity: String,
}
