use derivative::Derivative;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_size_limits::{AssetFilterFn, SizeLimitsPluginOptions};

#[derive(Derivative)]
#[derivative(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSizeLimitsPluginOptions {
  #[derivative(Debug = "ignore")]
  #[napi(ts_type = "(assetFilename: string) => boolean")]
  pub asset_filter: Option<ThreadsafeFunction<String, bool>>,
  #[napi(ts_type = "\"error\" | \"warning\"")]
  pub hints: Option<String>,
  pub max_asset_size: Option<f64>,
  pub max_entrypoint_size: Option<f64>,
}

impl From<RawSizeLimitsPluginOptions> for SizeLimitsPluginOptions {
  fn from(value: RawSizeLimitsPluginOptions) -> Self {
    SizeLimitsPluginOptions {
      asset_filter: value.asset_filter.map(|asset_filter| {
        let asset_filter_fn: AssetFilterFn = Box::new(move |name| {
          let f = asset_filter.clone();

          Box::pin(async move { f.call(name.to_owned()).await })
        });
        asset_filter_fn
      }),
      hints: value.hints,
      max_asset_size: value.max_asset_size,
      max_entrypoint_size: value.max_entrypoint_size,
    }
  }
}
