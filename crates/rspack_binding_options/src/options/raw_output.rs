use napi_derive::napi;
use rspack_core::OutputOptions;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: String,
  pub public_path: String,
  pub asset_module_filename: String,
  pub filename: String,
  pub chunk_filename: String,
  pub css_filename: String,
  pub css_chunk_filename: String,
  pub unique_name: String,
  pub library: Option<String>,
  pub strict_module_error_handling: bool,
  /* pub entry_filename: Option<String>,
   * pub source_map: Option<String>, */
}

impl From<RawOutputOptions> for OutputOptions {
  fn from(value: RawOutputOptions) -> Self {
    Self {
      path: value.path.into(),
      public_path: value.public_path.into(),
      asset_module_filename: value.asset_module_filename.into(),
      unique_name: value.unique_name.into(),
      filename: value.filename.into(),
      chunk_filename: value.chunk_filename.into(),
      css_filename: value.css_filename.into(),
      css_chunk_filename: value.css_chunk_filename.into(),
      library: value.library,
      strict_module_error_handling: value.strict_module_error_handling,
    }
  }
}
