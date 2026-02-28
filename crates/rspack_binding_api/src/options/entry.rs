use napi::Either;
use napi_derive::napi;
use rspack_core::{EntryOptions, EntryRuntime};

use super::library::JsLibraryOptions;
use crate::{
  filename::JsFilename,
  raw_options::{RawChunkLoading, RawWasmLoading},
};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct JsEntryPluginOptions {
  pub context: String,
  pub entry: String,
  pub options: JsEntryOptions,
}

pub type JsEntryRuntime = Either<bool, String>;
pub struct JsEntryRuntimeWrapper(pub JsEntryRuntime);

impl From<JsEntryRuntimeWrapper> for EntryRuntime {
  fn from(value: JsEntryRuntimeWrapper) -> Self {
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
pub struct JsEntryOptions {
  pub name: Option<String>,
  #[napi(ts_type = "false | string")]
  pub runtime: Option<JsEntryRuntime>,
  #[napi(ts_type = "false | string")]
  pub chunk_loading: Option<RawChunkLoading>,
  #[napi(ts_type = "false | string")]
  pub wasm_loading: Option<RawWasmLoading>,
  pub async_chunks: Option<bool>,
  #[napi(ts_type = "\"auto\" | JsFilename")]
  pub public_path: Option<JsFilename>,
  pub base_uri: Option<String>,
  pub filename: Option<JsFilename>,
  pub library: Option<JsLibraryOptions>,
  pub depend_on: Option<Vec<String>>,
  pub layer: Option<String>,
}

impl From<JsEntryOptions> for EntryOptions {
  fn from(value: JsEntryOptions) -> Self {
    Self {
      name: value.name,
      runtime: value.runtime.map(|r| JsEntryRuntimeWrapper(r).into()),
      chunk_loading: value.chunk_loading.map(Into::into),
      wasm_loading: value.wasm_loading.map(Into::into),
      async_chunks: value.async_chunks,
      public_path: value.public_path.map(Into::into),
      base_uri: value.base_uri,
      filename: value.filename.map(Into::into),
      library: value.library.map(Into::into),
      depend_on: value.depend_on,
      layer: value.layer,
    }
  }
}
