use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use anyhow::Context;
use serde::Deserialize;

#[cfg(feature = "node-api")]
use napi_derive::napi;

use rspack_core::{
  CompilerOptionsBuilder, Filename, OutputOptions, PublicPath, EXT_PLACEHOLDER, ID_PLACEHOLDER,
  NAME_PLACEHOLDER,
};

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
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: Option<String>,
  pub public_path: Option<String>,
  pub asset_module_filename: Option<String>,
  // todo support for function
  pub filename: Option<String>,
  pub chunk_filename: Option<String>,
  pub unique_name: Option<String>,
  /* pub entry_filename: Option<String>,
   * pub source_map: Option<String>, */
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawOutputOptions {
  pub path: Option<String>,
  pub public_path: Option<String>,
  pub asset_module_filename: Option<String>,
  // todo support for function
  pub filename: Option<String>,
  pub chunk_filename: Option<String>,
  pub unique_name: Option<String>,
  /* pub entry_filename: Option<String>,
   * pub source_map: Option<String>, */
}

impl RawOption<OutputOptions> for RawOutputOptions {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<OutputOptions> {
    // Align with https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/config/defaults.js#L750
    let filename = self
      .filename
      .unwrap_or(format!("{}{}", NAME_PLACEHOLDER, ".js",));

    let chunk_filename = self.chunk_filename.unwrap_or_else(|| {
      // Align with https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/config/defaults.js#L754
      let has_name = filename.contains(NAME_PLACEHOLDER);
      let has_id = filename.contains(ID_PLACEHOLDER);
      let has_chunk_hash = filename.contains("[chunkhash]");
      let hash_content_hash = filename.contains("[contenthash]");
      if has_name || has_id || has_chunk_hash || hash_content_hash {
        filename.clone()
      } else {
        let re = regex::Regex::new(r#"(^|\\/)([^/]*(?:\\?|$))"#).unwrap();
        let captures = re.captures(&filename);
        format!(
          "{}[id].{}",
          captures
            .as_ref()
            .unwrap()
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or(""),
          captures
            .as_ref()
            .unwrap()
            .get(2)
            .map(|m| m.as_str())
            .unwrap_or(""),
        )
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
      .unwrap_or_else(|| format!("assets/{}", filename));
    Ok(OutputOptions {
      path,
      asset_module_filename: Filename::from_str(&asset_module_filename)?,
      filename: Filename::from_str(&filename)?,
      chunk_filename: Filename::from_str(&chunk_filename)?,
      unique_name,
      public_path: PublicPath::from_str(&public_path)?,
    })
  }

  fn fallback_value(_: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
