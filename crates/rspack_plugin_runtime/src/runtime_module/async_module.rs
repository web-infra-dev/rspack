use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct AsyncRuntimeModule;

impl RuntimeModule for AsyncRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/async_module.js")).boxed()
  }

  fn identifier(&self) -> String {
    "webpack/runtime/async_module".into()
  }
}
