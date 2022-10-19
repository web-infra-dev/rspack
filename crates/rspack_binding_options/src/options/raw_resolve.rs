use std::collections::HashMap;

use crate::RawOption;
#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{AliasMap, CompilerOptionsBuilder, Resolve};
use serde::Deserialize;

pub type AliasValue = serde_json::Value;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
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
  pub symlinks: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawResolveOptions {
  pub prefer_relative: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub browser_field: Option<bool>,
  pub condition_names: Option<Vec<String>>,
  #[serde(serialize_with = "ordered_map")]
  pub alias: Option<HashMap<String, AliasValue>>,
  pub symlinks: Option<bool>,
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
    let alias = if let Some(alias) = self.alias {
      let mut temp = vec![];
      for (key, value) in alias {
        if let Some(s) = value.as_str() {
          temp.push((key, AliasMap::Target(s.to_string())))
        } else if let Some(b) = value.as_bool() {
          if b {
            return Err(anyhow::Error::msg(format!(
              "Alias should not be true in {}",
              key
            )));
          } else {
            temp.push((key, AliasMap::Ignored))
          }
        } else {
          return Err(anyhow::Error::msg(format!(
            "Alias should be false or string in {}",
            key
          )));
        }
      }
      Some(temp)
    } else {
      None
    };

    Ok(Resolve {
      prefer_relative,
      extensions,
      browser_field,
      main_fields,
      main_files,
      condition_names,
      alias,
      symlinks,
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
