use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{Compilation, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH};

#[derive(Debug, Default)]
pub struct AsyncWasmRuntimeModule;

impl RuntimeModule for AsyncWasmRuntimeModule {
  fn identifier(&self) -> String {
    "rspack/runtime/wasm loading".into()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/async-wasm-fetch-loading.js")).boxed()
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}
