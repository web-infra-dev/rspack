use std::{path::Path, str::FromStr};

use rspack_core::{
  Filename, Mode, OutputOptions, PublicPath, EXT_PLACEHOLDER, ID_PLACEHOLDER, NAME_PLACEHOLDER,
};
use serde::Deserialize;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(feature = "test"), napi(object))]
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

impl RawOutputOptions {
  pub fn normalize(self, _mode: &Mode, context: &str) -> OutputOptions {
    // let is_prod = matches!(mode, Mode::Production);
    let filename = self.filename.unwrap_or(format!(
      "{}{}{}",
      NAME_PLACEHOLDER,
      // todo need add hash
      // if is_prod {
      //   CONTENT_PLACEHOLDER
      // } else {
      //   HASH_PLACEHOLDER
      // },
      "",
      EXT_PLACEHOLDER
    ));

    let chunk_filename = self
      .chunk_filename
      .unwrap_or_else(|| filename.replace(NAME_PLACEHOLDER, ID_PLACEHOLDER));
    let path = self.path.unwrap_or_else(|| {
      Path::new(context)
        .join("dist")
        .to_string_lossy()
        .to_string()
    });
    // todo unique name needs to be determined by package.name
    let unique_name = self
      .unique_name
      .unwrap_or_else(|| String::from("__rspack_runtime__"));
    let public_path = self.public_path.unwrap_or_else(|| String::from("/"));
    let asset_module_filename = self
      .asset_module_filename
      .unwrap_or_else(|| format!("assets/{}", filename));
    // let public_path =
    OutputOptions {
      path,
      asset_module_filename: Filename::from_str(&asset_module_filename).unwrap(),
      filename: Filename::from_str(&filename).unwrap(),
      chunk_filename: Filename::from_str(&chunk_filename).unwrap(),
      unique_name,
      public_path: PublicPath::from_str(&public_path).unwrap(),
    }
  }
}
