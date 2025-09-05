use napi::Either;
use napi_derive::napi;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_plugin_swc_js_minimizer::{
  ExtractComments, MinimizerOptions, OptionWrapper, PluginOptions,
};
use serde::de::DeserializeOwned;
use swc_core::base::BoolOrDataConfig;

use crate::asset_condition::{RawAssetConditions, into_asset_conditions};

#[derive(Debug)]
#[napi(object)]
pub struct RawExtractComments {
  pub banner: Option<Either<String, bool>>,
  pub condition: Option<String>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSwcJsMinimizerRspackPluginOptions {
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub test: Option<RawAssetConditions>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub include: Option<RawAssetConditions>,
  #[napi(ts_type = "string | RegExp | (string | RegExp)[]")]
  pub exclude: Option<RawAssetConditions>,
  pub extract_comments: Option<RawExtractComments>,
  pub minimizer_options: RawSwcJsMinimizerOptions,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSwcJsMinimizerOptions {
  pub ecma: serde_json::Value,
  pub compress: serde_json::Value,
  pub mangle: serde_json::Value,
  pub format: serde_json::Value,
  pub module: Option<bool>,
  pub minify: Option<bool>,
}

fn try_deserialize_into<T>(value: serde_json::Value) -> Result<T>
where
  T: DeserializeOwned,
{
  serde_json::from_value(value).to_rspack_result()
}

fn into_extract_comments(c: Option<RawExtractComments>) -> Option<ExtractComments> {
  let c = c?;
  let condition = c.condition?;
  let banner = match c.banner {
    Some(banner) => match banner {
      Either::A(s) => OptionWrapper::Custom(s),
      Either::B(b) => {
        if b {
          OptionWrapper::Default
        } else {
          OptionWrapper::Disabled
        }
      }
    },
    None => OptionWrapper::Default,
  };

  Some(ExtractComments { condition, banner })
}

impl TryFrom<RawSwcJsMinimizerRspackPluginOptions> for PluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawSwcJsMinimizerRspackPluginOptions) -> Result<Self> {
    let compress = try_deserialize_into::<
      BoolOrDataConfig<rspack_plugin_swc_js_minimizer::TerserCompressorOptions>,
    >(value.minimizer_options.compress)?
    .or(|| BoolOrDataConfig::from_bool(true));
    let mangle = try_deserialize_into::<
      BoolOrDataConfig<rspack_plugin_swc_js_minimizer::MangleOptions>,
    >(value.minimizer_options.mangle)?
    .or(|| BoolOrDataConfig::from_bool(true));

    let ecma = try_deserialize_into::<rspack_plugin_swc_js_minimizer::TerserEcmaVersion>(
      value.minimizer_options.ecma,
    )?;

    Ok(Self {
      extract_comments: into_extract_comments(value.extract_comments),
      test: value.test.map(into_asset_conditions),
      include: value.include.map(into_asset_conditions),
      exclude: value.exclude.map(into_asset_conditions),
      minimizer_options: MinimizerOptions {
        compress,
        mangle,
        ecma,
        format: try_deserialize_into(value.minimizer_options.format)?,
        module: value.minimizer_options.module,
        minify: value.minimizer_options.minify,
        ..Default::default()
      },
    })
  }
}
