use derive_more::Debug;
use napi_derive::napi;
use rspack_plugin_size_limits::{AssetFilterFn, SizeLimitsPluginOptions};

use crate::compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSizeLimitsPluginOptions {
  pub async_chunk_waterfalls: Option<bool>,
  #[debug(skip)]
  #[napi(ts_type = "(assetFilename: string) => boolean")]
  pub asset_filter: Option<ThreadsafeFunction<String, bool>>,
  pub embedded_source_maps: Option<bool>,
  #[napi(ts_type = "\"error\" | \"warning\"")]
  pub hints: Option<String>,
  pub inlined_assets: Option<bool>,
  pub max_asset_size: Option<f64>,
  pub max_entrypoint_size: Option<f64>,
  pub top_level_this: Option<bool>,
}

impl From<RawSizeLimitsPluginOptions> for SizeLimitsPluginOptions {
  fn from(value: RawSizeLimitsPluginOptions) -> Self {
    SizeLimitsPluginOptions {
      async_chunk_waterfalls: value.async_chunk_waterfalls.unwrap_or(false),
      asset_filter: value.asset_filter.map(|asset_filter| {
        let asset_filter_fn: AssetFilterFn = Box::new(move |name| {
          let f = asset_filter.clone();

          Box::pin(async move { f.call_with_sync(name.to_owned()).await })
        });
        asset_filter_fn
      }),
      embedded_source_maps: value.embedded_source_maps.unwrap_or(false),
      hints: value.hints,
      inlined_assets: value.inlined_assets.unwrap_or(false),
      max_asset_size: value.max_asset_size,
      max_entrypoint_size: value.max_entrypoint_size,
      top_level_this: value.top_level_this.unwrap_or(false),
    }
  }
}
