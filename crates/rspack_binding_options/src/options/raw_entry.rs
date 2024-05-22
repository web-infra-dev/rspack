use napi::Either;
use napi_derive::napi;
use rspack_binding_values::JsFilename;
use rspack_core::{EntryOptions, EntryRuntime};

use crate::RawLibraryOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawEntryPluginOptions {
  pub context: String,
  pub entry: String,
  pub options: RawEntryOptions,
}

pub type RawEntryRuntime = Either<bool, String>;
pub struct RawEntryRuntimeWrapper(pub RawEntryRuntime);

impl From<RawEntryRuntimeWrapper> for EntryRuntime {
  fn from(value: RawEntryRuntimeWrapper) -> Self {
    match value.0 {
      Either::A(b) => {
        assert!(!b, "RawEntryRuntime should be false or string");
        Self::False
      }
      Either::B(s) => Self::String(s),
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawEntryOptions {
  pub name: Option<String>,
  #[napi(ts_type = "false | string")]
  pub runtime: Option<RawEntryRuntime>,
  pub chunk_loading: Option<String>,
  pub async_chunks: Option<bool>,
  pub public_path: Option<String>,
  pub base_uri: Option<String>,
  #[napi(ts_type = "string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string)")]
  pub filename: Option<JsFilename>,
  pub library: Option<RawLibraryOptions>,
  pub depend_on: Option<Vec<String>>,
}

impl From<RawEntryOptions> for EntryOptions {
  fn from(value: RawEntryOptions) -> Self {
    Self {
      name: value.name,
      runtime: value.runtime.map(|r| RawEntryRuntimeWrapper(r).into()),
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
