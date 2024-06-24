use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncRuntimeModule {
  id: Identifier,
}
impl Default for AsyncRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/async_module"))
  }
}

impl RuntimeModule for AsyncRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/async_module.js")).boxed())
  }

  fn name(&self) -> Identifier {
    self.id
  }
}
