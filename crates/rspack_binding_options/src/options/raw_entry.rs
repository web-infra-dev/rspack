use napi_derive::napi;
use rspack_core::EntryOptions;
use serde::Deserialize;

use crate::RawLibraryOptions;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawEntryPluginOptions {
  pub context: String,
  pub entry: String,
  pub options: RawEntryOptions,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawEntryOptions {
  pub name: Option<String>,
  pub runtime: Option<String>,
  pub chunk_loading: Option<String>,
  pub async_chunks: Option<bool>,
  pub public_path: Option<String>,
  pub base_uri: Option<String>,
  pub filename: Option<String>,
  pub library: Option<RawLibraryOptions>,
  pub depend_on: Option<Vec<String>>,
}

impl From<RawEntryOptions> for EntryOptions {
  fn from(value: RawEntryOptions) -> Self {
    Self {
      name: value.name,
      runtime: value.runtime,
      chunk_loading: value.chunk_loading.as_deref().map(Into::into),
      async_chunks: value.async_chunks,
      public_path: value.public_path.map(Into::into),
      base_uri: value.base_uri,
      filename: value.filename.map(Into::into),
      library: value.library.map(Into::into),
      depend_on: value.depend_on.map(Into::into),
    }
  }
}
