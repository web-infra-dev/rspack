use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct GlobalRuntimeModule {
  id: Identifier,
}

impl Default for GlobalRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/global"))
  }
}

impl RuntimeModule for GlobalRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/global.js")).boxed())
  }
}
