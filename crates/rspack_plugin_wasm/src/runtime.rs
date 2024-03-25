use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{
  get_filename_without_hash_length, impl_runtime_module, ChunkUkey, Compilation, PathData,
  RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_util::infallible::ResultInfallibleExt as _;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct AsyncWasmLoadingRuntimeModule {
  generate_load_binary_code: String,
  id: Identifier,
  supports_streaming: bool,
  chunk: ChunkUkey,
}

impl AsyncWasmLoadingRuntimeModule {
  pub fn new(
    generate_load_binary_code: String,
    supports_streaming: bool,
    chunk: ChunkUkey,
  ) -> Self {
    Self {
      generate_load_binary_code,
      id: Identifier::from("webpack/runtime/async_wasm_loading"),
      supports_streaming,
      chunk,
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for AsyncWasmLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let (fake_filename, hash_len_map) =
      get_filename_without_hash_length(&compilation.options.output.webassembly_module_filename);

    // Even use content hash when [hash] in webpack
    let hash = match hash_len_map
      .get("[contenthash]")
      .or(hash_len_map.get("[hash]"))
    {
      Some(hash_len) => format!("\" + wasmModuleHash.slice(0, {}) + \"", hash_len),
      None => "\" + wasmModuleHash + \"".to_string(),
    };

    let chunk = compilation.chunk_by_ukey.expect_get(&self.chunk);
    let path = compilation
      .get_path(
        &fake_filename,
        PathData::default()
          .hash(&hash)
          .content_hash(&hash)
          .id("\" + wasmModuleId + \"")
          .runtime(&chunk.runtime),
      )
      .always_ok();
    Ok(
      RawSource::from(get_async_wasm_loading(
        &self
          .generate_load_binary_code
          .replace("$PATH", &format!("\"{}\"", path)),
        self.supports_streaming,
      ))
      .boxed(),
    )
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

fn get_async_wasm_loading(req: &str, supports_streaming: bool) -> String {
  let fallback_code = r#"
          .then(function(x) { return x.arrayBuffer();})
          .then(function(bytes) { return WebAssembly.instantiate(bytes, importsObj);})
          .then(function(res) { return Object.assign(exports, res.instance.exports);});
"#;

  let streaming_code = r#"
      return req.then(function(res) {
        if (typeof WebAssembly.instantiateStreaming === "function") {
          return WebAssembly.instantiateStreaming(res, importsObj)
            .then(
              function(res) { return Object.assign(exports, res.instance.exports);},
              function(e) {
                if(res.headers.get("Content-Type") !== "application/wasm") {
                  console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                  return fallback();
                }
                throw e;
              }
            );
        }
        return fallback();
      });
"#;

  if supports_streaming {
    format!(
      r#"
    __webpack_require__.v = function(exports, wasmModuleId, wasmModuleHash, importsObj) {{
      var req = {req};
      var fallback = function() {{
        return req{fallback_code}
      }}
      {streaming_code}
    }};
"#
    )
  } else {
    let req = req.trim_end_matches(';');
    format!(
      r#"
    __webpack_require__.v = function(exports, wasmModuleId, wasmModuleHash, importsObj) {{
      return {req}{fallback_code}
    }};
      "#
    )
  }
}
