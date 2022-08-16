use napi_derive::napi;
use serde::Deserialize;

use rspack_core::{
  AssetParserDataUrlOption, AssetParserOptions, CompilerOptionsBuilder, ModuleOptions,
  ParserOptions,
};

use crate::RawOption;

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawModuleRule {}

#[derive(Debug, Clone, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawAssetParserDataUrlOption {
  pub max_size: Option<u32>,
}
#[derive(Debug, Clone, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawAssetParserOptions {
  pub data_url_condition: Option<RawAssetParserDataUrlOption>,
}

#[derive(Debug, Clone, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawParserOptions {
  pub asset: Option<RawAssetParserOptions>,
}

#[derive(Default, Debug, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
  pub parser: Option<RawParserOptions>,
}

impl RawOption<Option<ModuleOptions>> for RawModuleOptions {
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<Option<ModuleOptions>> {
    // FIXME: temporary implementation
    Ok(Some(ModuleOptions {
      rules: vec![],
      parser: self.parser.map(|x| ParserOptions {
        asset: x.asset.map(|y| AssetParserOptions {
          data_url_condition: y.data_url_condition.map(|a| AssetParserDataUrlOption {
            max_size: a.max_size,
          }),
        }),
      }),
    }))
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawModuleOptions {
      rules: vec![],
      parser: None,
    }
  }
}
