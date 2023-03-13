use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};
use rspack_core::{Compilation, RuntimeModule};

#[derive(Debug, Default)]
pub struct AsyncWasmRuntimeModule;

impl RuntimeModule for AsyncWasmRuntimeModule {
  fn identifier(&self) -> String {
    "wasm loading".into()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/async-wasm-fetch-loading.js").to_string()).boxed()
  }
}
