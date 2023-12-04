use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{
  get_filename_without_hash_length, impl_runtime_module, ChunkUkey, Compilation, PathData,
  RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;

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
    }
  }
}

impl RuntimeModule for AsyncWasmLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, compilation: &Compilation) -> BoxSource {
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

    let chunk = compilation
      .chunk_by_ukey
      .get(&self.chunk)
      .expect("chunk not found");
    let path = compilation.get_path(
      &fake_filename,
      PathData::default()
        .hash(&hash)
        .content_hash(&hash)
        .id("\" + wasmModuleId + \"")
        .runtime(&chunk.runtime),
    );
    RawSource::from(get_async_wasm_loading(
      &self
        .generate_load_binary_code
        .replace("$PATH", &format!("\"{}\"", path)),
      self.supports_streaming,
    ))
    .boxed()
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

impl_runtime_module!(AsyncWasmLoadingRuntimeModule);

fn get_async_wasm_loading(req: &str, supports_streaming: bool) -> String {
  let streaming_code = if supports_streaming {
    r#"
    if (typeof WebAssembly.instantiateStreaming === 'function') {
      return WebAssembly.instantiateStreaming(req, importsObj).then(function(res) {
        return Object.assign(exports, res.instance.exports);
      });
    }
    "#
  } else {
    "// no support for streaming compilation"
  };
  format!(
    r#"
    __webpack_require__.v = function(exports, wasmModuleId, wasmModuleHash, importsObj) {{
      var req = {req}
      {streaming_code}
      return req
        .then(function(x) {{
          return x.arrayBuffer();
        }})
        .then(function(bytes) {{
          return WebAssembly.instantiate(bytes, importsObj);
        }})
        .then(function(res) {{
          return Object.assign(exports, res.instance.exports);
        }});
    }};
    "#
  )
}
