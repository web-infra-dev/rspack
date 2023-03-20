use std::path::PathBuf;

use napi_derive::napi;
use rspack_core::{CopyPluginConfig, GlobOptions, Pattern, ToType};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawPattern {
  pub from: String,
  pub to: Option<String>,
  pub context: Option<String>,
  pub to_type: Option<String>,
  pub no_error_on_missing: bool,
  pub force: bool,
  pub priority: i32,
  pub glob_options: RawGlobOptions,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawGlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
  pub ignore: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCopyConfig {
  pub patterns: Vec<RawPattern>,
}

impl From<RawPattern> for Pattern {
  fn from(value: RawPattern) -> Self {
    let RawPattern {
      from,
      to,
      context,
      to_type,
      no_error_on_missing,
      force,
      priority,
      glob_options,
    } = value;

    Self {
      from,
      to,
      context: context.map(PathBuf::from),
      to_type: if let Some(to_type) = to_type {
        match to_type.to_lowercase().as_str() {
          "dir" => Some(ToType::Dir),
          "file" => Some(ToType::File),
          "template" => Some(ToType::Template),
          _ => {
            //TODO how should we handle wrong input ?
            None
          }
        }
      } else {
        None
      },
      no_error_on_missing,
      info: None,
      force,
      priority,
      glob_options: GlobOptions {
        case_sensitive_match: glob_options.case_sensitive_match,
        dot: glob_options.dot,
        ignore: glob_options.ignore.map(|ignore| {
          ignore
            .into_iter()
            .map(|filter| glob::Pattern::new(filter.as_ref()).expect("Invalid pattern option"))
            .collect()
        }),
      },
    }
  }
}

impl From<RawCopyConfig> for CopyPluginConfig {
  fn from(val: RawCopyConfig) -> Self {
    Self {
      patterns: val.patterns.into_iter().map(Into::into).collect(),
    }
  }
}
