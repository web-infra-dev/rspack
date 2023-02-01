use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{AliasMap, CompilerOptionsBuilder, Resolve};
use serde::Deserialize;

use crate::RawOption;

pub type AliasValue = serde_json::Value;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawResolveOptions {
  pub prefer_relative: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub browser_field: Option<bool>,
  pub condition_names: Option<Vec<String>>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, string | false>")]
  pub alias: Option<HashMap<String, AliasValue>>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, string | false>")]
  pub fallback: Option<HashMap<String, AliasValue>>,
  pub symlinks: Option<bool>,
  pub ts_config_path: Option<String>,
  pub modules: Option<Vec<String>>,
}

fn normalize_alias(
  alias: Option<HashMap<String, AliasValue>>,
) -> anyhow::Result<Option<Vec<(String, AliasMap)>>> {
  alias
    .map(|alias| {
      alias
        .into_iter()
        .map(|(key, value)| {
          if let Some(s) = value.as_str() {
            Ok((key, AliasMap::Target(s.to_string())))
          } else if let Some(b) = value.as_bool() {
            if b {
              Err(anyhow::Error::msg(format!(
                "Alias should not be true in {key}"
              )))
            } else {
              Ok((key, AliasMap::Ignored))
            }
          } else {
            Err(anyhow::Error::msg(format!(
              "Alias should be false or string in {key}"
            )))
          }
        })
        .collect::<anyhow::Result<_>>()
    })
    .map_or(Ok(None), |v| v.map(Some))
}

impl RawOption<Resolve> for RawResolveOptions {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Resolve> {
    let prefer_relative = self.prefer_relative;
    let extensions = self.extensions;
    let browser_field = self.browser_field;
    let main_files = self.main_files;
    let main_fields = self.main_fields;
    let condition_names = self.condition_names;
    let symlinks = self.symlinks;
    let alias = normalize_alias(self.alias)?;
    let fallback = normalize_alias(self.fallback)?;
    let modules = self.modules;
    let tsconfig = self.ts_config_path.map(std::path::PathBuf::from);
    Ok(Resolve {
      modules,
      prefer_relative,
      extensions,
      browser_field,
      main_fields,
      main_files,
      condition_names,
      alias,
      symlinks,
      tsconfig,
      fallback,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
