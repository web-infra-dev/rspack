use crate::RawOption;
use napi_derive::napi;
use rspack_core::{
  CompilerOptionsBuilder, Filename, OutputOptions, PublicPath, EXT_PLACEHOLDER, ID_PLACEHOLDER,
  NAME_PLACEHOLDER,
};
use serde::Deserialize;
use std::{path::Path, str::FromStr};

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
  pub unique_name: Option<String>,
  /* pub entry_filename: Option<String>,
   * pub source_map: Option<String>, */
}

impl RawOption<OutputOptions> for RawOutputOptions {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<OutputOptions> {
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
      Path::new(options.context.as_ref().unwrap())
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
