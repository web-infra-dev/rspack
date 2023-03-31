use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{Compilation, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH};
use rspack_identifier::Identifier;
use rspack_plugin_runtime::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct AsyncWasmLoadingRuntimeModule {
  generate_load_binary_code: String,
  id: Identifier,
}

impl AsyncWasmLoadingRuntimeModule {
  pub fn new(generate_load_binary_code: String) -> Self {
    Self {
      generate_load_binary_code,
      id: Identifier::from("rspack/runtime/wasm loading"),
    }
  }
}

impl RuntimeModule for AsyncWasmLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    let path = "wasmModuleHash";
    RawSource::from(include_str!("runtime/async-wasm-loading.js").replace(
      "$REQ$",
      &self.generate_load_binary_code.replace("$PATH", path),
    ))
    .boxed()
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(AsyncWasmLoadingRuntimeModule);
