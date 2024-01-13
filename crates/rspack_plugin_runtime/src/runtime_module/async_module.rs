use rspack_common::SourceMapKind;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct AsyncRuntimeModule {
  id: Identifier,
}
impl Default for AsyncRuntimeModule {
  fn default() -> Self {
    AsyncRuntimeModule {
      id: Identifier::from("webpack/runtime/async_module"),
      source_map_option: SourceMapKind::None,
    }
  }
}

impl RuntimeModule for AsyncRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/async_module.js")).boxed()
  }

  fn name(&self) -> Identifier {
    self.id
  }
}
