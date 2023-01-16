use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use anyhow::Context;
use napi_derive::napi;
use rspack_core::{
  CompilerOptionsBuilder, Filename, OutputOptions, PublicPath, CHUNK_HASH_PLACEHOLDER,
  CONTENT_HASH_PLACEHOLDER, EXT_PLACEHOLDER, HASH_PLACEHOLDER, ID_PLACEHOLDER, NAME_PLACEHOLDER,
  QUERY_PLACEHOLDER,
};
use serde::Deserialize;

use crate::RawOption;

pub fn generate_path(path: Option<String>, context: &Path) -> PathBuf {
  match path {
    Some(path) => {
      let path = PathBuf::from(&path);
      if path.is_absolute() {
        path
      } else {
        context.join(path)
      }
    }
    None => context.join("dist"),
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: Option<String>,
  pub public_path: Option<String>,
  pub asset_module_filename: Option<String>,
  // todo support for function
  pub filename: Option<String>,
  pub chunk_filename: Option<String>,
  pub css_filename: Option<String>,
  pub css_chunk_filename: Option<String>,
  pub unique_name: Option<String>,
  pub library: Option<String>,
  pub strict_module_error_handling: Option<bool>,
  /* pub entry_filename: Option<String>,
   * pub source_map: Option<String>, */
}

impl RawOption<OutputOptions> for RawOutputOptions {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<OutputOptions> {
    // Align with https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/config/defaults.js#L750
    let filename = self
      .filename
      .clone()
      .unwrap_or(format!("{}{}", NAME_PLACEHOLDER, ".js",));

    let chunk_filename = self.chunk_filename.clone().unwrap_or_else(|| {
      // Align with https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/config/defaults.js#L754
      let has_name = filename.contains(NAME_PLACEHOLDER);
      let has_id = filename.contains(ID_PLACEHOLDER);
      let has_chunk_hash = filename.contains(CHUNK_HASH_PLACEHOLDER);
      let hash_content_hash = filename.contains(CONTENT_HASH_PLACEHOLDER);
      if has_name || has_id || has_chunk_hash || hash_content_hash {
        filename.clone()
      } else {
        let re = regex::Regex::new(r#"(^|\\/)([^/]*(?:\\?|$))"#).expect("should be valid regex");
        let captures = re.captures(&filename);
        format!(
          "{}[id].{}",
          captures
            .as_ref()
            .expect("value should be captrued")
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or(""),
          captures
            .as_ref()
            .expect("value should be captrued")
            .get(2)
            .map(|m| m.as_str())
            .unwrap_or(""),
        )
      }
    });

    let css_filename = self.css_filename.unwrap_or_else(|| {
      if let Some(filename) = self.filename.as_ref() {
        let re = regex::Regex::new(r#"\.[mc]?js(\?|$)"#).expect("should be valid regex");
        re.replace(filename, ".css$1").to_string()
      } else {
        format!("{}{}", ID_PLACEHOLDER, ".css")
      }
    });

    let css_chunk_filename = self.css_chunk_filename.unwrap_or_else(|| {
      if let Some(filename) = self.chunk_filename.as_ref() {
        let re = regex::Regex::new(r#"\.[mc]?js(\?|$)"#).expect("should be valid regex");
        re.replace(filename, ".css$1").to_string()
      } else {
        format!("{}{}", ID_PLACEHOLDER, ".css")
      }
    });
    let path = generate_path(
      self.path,
      options.context.as_ref().context("should have context")?,
    );
    // todo unique name needs to be determined by package.name
    let unique_name = self
      .unique_name
      .unwrap_or_else(|| String::from("__rspack_runtime__"));
    let public_path = self.public_path.unwrap_or_else(|| String::from("/"));
    let asset_module_filename = self
      .asset_module_filename
      .unwrap_or_else(|| format!("{HASH_PLACEHOLDER}{EXT_PLACEHOLDER}{QUERY_PLACEHOLDER}"));
    let library = self.library;
    Ok(OutputOptions {
      path,
      asset_module_filename: Filename::from_str(&asset_module_filename)?,
      filename: Filename::from_str(&filename)?,
      chunk_filename: Filename::from_str(&chunk_filename)?,
      css_filename: Filename::from_str(&css_filename)?,
      css_chunk_filename: Filename::from_str(&css_chunk_filename)?,
      unique_name,
      public_path: PublicPath::from_str(&public_path)?,
      library,
      strict_module_error_handling: self.strict_module_error_handling,
    })
  }

  fn fallback_value(_: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
