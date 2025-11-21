use std::sync::Arc;

use derive_more::Debug;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_sri::{
  IntegrityCallbackData, SubresourceIntegrityHashFunction, SubresourceIntegrityPluginOptions,
};

#[derive(Debug)]
#[napi(
  object,
  object_to_js = false,
  js_name = "RawSubresourceIntegrityPluginOptions"
)]
pub struct RawSubresourceIntegrityPluginOptions {
  #[debug(skip)]
  #[napi(ts_type = "(data: RawIntegrityData) => void")]
  pub integrity_callback: Option<ThreadsafeFunction<RawIntegrityData, ()>>,
  pub hash_func_names: Vec<String>,
  #[napi(ts_type = "\"JavaScript\" | \"Native\" | \"Disabled\"")]
  pub html_plugin: String,
}

impl TryFrom<RawSubresourceIntegrityPluginOptions> for SubresourceIntegrityPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(options: RawSubresourceIntegrityPluginOptions) -> Result<Self, rspack_error::Error> {
    let html_plugin = options.html_plugin.try_into()?;
    if options.hash_func_names.is_empty() {
      return Err(rspack_error::Error::error(
        "Expect at least one SRI hash function name.".to_string(),
      ));
    }
    let hash_func_names = options
      .hash_func_names
      .into_iter()
      .map(SubresourceIntegrityHashFunction::try_from)
      .collect::<Result<Vec<_>, rspack_error::Error>>()?;
    Ok(Self {
      integrity_callback: if let Some(func) = options.integrity_callback {
        Some(Arc::new(move |data| {
          let func = func.clone();
          Box::pin(async move { func.call_with_sync(data.into()).await })
        }))
      } else {
        None
      },
      hash_func_names,
      html_plugin,
    })
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
