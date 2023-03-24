use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{RuntimeGlobals, Compilation, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH};
use rspack_identifier::Identifier;
use rspack_plugin_runtime::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct AsyncWasmRuntimeModule {
  id: Identifier,
}

impl Default for AsyncWasmRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("rspack/runtime/wasm loading"),
    }
  }
}

impl RuntimeModule for AsyncWasmRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/async-wasm-loading.js").replace(
      "$REQ$",
      &format!("fetch({}+wasmModuleHash)", RuntimeGlobals::PUBLIC_PATH),
    ))
    .boxed()
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(AsyncWasmRuntimeModule);
