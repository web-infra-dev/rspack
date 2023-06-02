use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{Compilation, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH};
use rspack_identifier::Identifier;
use rspack_plugin_runtime::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct AsyncWasmLoadingRuntimeModule {
  generate_load_binary_code: String,
  id: Identifier,
  supports_streaming: bool,
}

impl AsyncWasmLoadingRuntimeModule {
  pub fn new(generate_load_binary_code: String, supports_streaming: bool) -> Self {
    Self {
      generate_load_binary_code,
      id: Identifier::from("rspack/runtime/wasm loading"),
      supports_streaming,
    }
  }
}

impl RuntimeModule for AsyncWasmLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    let path = "wasmModuleHash";
    RawSource::from(get_async_wasm_loading(
      &self.generate_load_binary_code.replace("$PATH", path),
      self.supports_streaming,
    ))
    .boxed()
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(AsyncWasmLoadingRuntimeModule);

fn get_async_wasm_loading(req: &str, supports_streaming: bool) -> String {
  let streaming_code = if supports_streaming {
    r#"if (typeof WebAssembly.instantiateStreaming === "function") {
  return WebAssembly.instantiateStreaming(req, importsObj).then(
    res => Object.assign(exports, res.instance.exports)
  );
}"#
  } else {
    "// no support for streaming compilation"
  };
  format!(
    r#"__webpack_require__.v = (exports, wasmModuleId, wasmModuleHash, importsObj) => {{
  var req = {req};
  {streaming_code}
  return req
    .then(x => x.arrayBuffer())
    .then(bytes => WebAssembly.instantiate(bytes, importsObj))
    .then(res => Object.assign(exports, res.instance.exports));
}};
"#
  )
}
