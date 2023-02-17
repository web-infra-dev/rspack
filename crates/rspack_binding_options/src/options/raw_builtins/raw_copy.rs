use std::path::PathBuf;

use napi_derive::napi;
use rspack_core::{CopyPluginConfig, GlobOptions, Pattern, ToType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawPattern {
  pub from: String,
  pub to: Option<String>,
  pub context: Option<String>,
  pub to_type: Option<String>,
  pub no_error_on_missing: Option<bool>,
  pub force: Option<bool>,
  pub priority: Option<i32>,
  pub glob_options: Option<RawGlobOptions>,
}

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawGlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawCopyConfig {
  pub patterns: Vec<RawPattern>,
}

impl From<RawCopyConfig> for CopyPluginConfig {
  fn from(val: RawCopyConfig) -> Self {
    Self {
      patterns: val
        .patterns
        .into_iter()
        .map(|pattern| -> Pattern {
          Pattern {
            from: pattern.from,
            to: pattern.to,
            context: pattern.context.map(PathBuf::from),
            to_type: if let Some(to_type) = pattern.to_type {
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
            no_error_on_missing: pattern.no_error_on_missing.unwrap_or(false),
            info: None,
            force: pattern.force.unwrap_or(false),
            priority: pattern.priority.unwrap_or(0),
            glob_options: pattern.glob_options.map(|opt| GlobOptions {
              case_sensitive_match: opt.case_sensitive_match,
              dot: opt.dot,
            }),
          }
        })
        .collect(),
    }
  }
}
