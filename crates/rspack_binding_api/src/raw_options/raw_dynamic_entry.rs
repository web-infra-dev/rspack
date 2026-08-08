use napi::{Either, bindgen_prelude::Promise};
use napi_derive::napi;
use rspack_core::EntryOptions;
use rspack_plugin_dynamic_entry::{DynamicEntryPluginOptions, EntryDynamicResult};

use crate::{
  compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction,
  filename::JsGenerationFilename,
  options::{entry::JsEntryRuntimeWrapper, library::JsLibraryOptions},
  raw_options::{RawChunkLoading, RawWasmLoading},
};

type JsEntryRuntime = Either<bool, String>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct JsDynamicEntryOptions {
  pub name: Option<String>,
  #[napi(ts_type = "false | string")]
  pub runtime: Option<JsEntryRuntime>,
  #[napi(ts_type = "false | string")]
  pub chunk_loading: Option<RawChunkLoading>,
  #[napi(ts_type = "false | string")]
  pub wasm_loading: Option<RawWasmLoading>,
  pub async_chunks: Option<bool>,
  #[napi(ts_type = "\"auto\" | JsFilename")]
  pub public_path: Option<JsGenerationFilename>,
  pub base_uri: Option<String>,
  #[napi(ts_type = "JsFilename")]
  pub filename: Option<JsGenerationFilename>,
  pub library: Option<JsLibraryOptions>,
  pub depend_on: Option<Vec<String>>,
  pub layer: Option<String>,
}

impl From<JsDynamicEntryOptions> for EntryOptions {
  fn from(value: JsDynamicEntryOptions) -> Self {
    Self {
      name: value.name,
      runtime: value
        .runtime
        .map(|runtime| JsEntryRuntimeWrapper(runtime).into()),
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

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawEntryDynamicResult {
  pub import: Vec<String>,
  #[napi(ts_type = "JsEntryOptions")]
  pub options: JsDynamicEntryOptions,
}

pub type RawEntryDynamic = ThreadsafeFunction<(), Promise<Vec<RawEntryDynamicResult>>>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDynamicEntryPluginOptions {
  pub context: String,
  #[napi(ts_type = "() => Promise<RawEntryDynamicResult[]>")]
  pub entry: RawEntryDynamic,
}

impl From<RawDynamicEntryPluginOptions> for DynamicEntryPluginOptions {
  fn from(opts: RawDynamicEntryPluginOptions) -> Self {
    Self {
      context: opts.context.into(),
      entry: Box::new(move || {
        let f = opts.entry.clone();
        Box::pin(async move {
          let raw_result = f.call_with_promise(()).await?;
          let result = raw_result
            .into_iter()
            .map(
              |RawEntryDynamicResult { import, options }| EntryDynamicResult {
                import,
                options: options.into(),
              },
            )
            .collect::<Vec<_>>();
          Ok(result)
        })
      }),
    }
  }
}
