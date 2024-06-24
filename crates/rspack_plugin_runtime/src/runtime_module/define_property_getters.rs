use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct DefinePropertyGettersRuntimeModule {
  id: Identifier,
}

impl Default for DefinePropertyGettersRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/define_property_getters"))
  }
}

impl RuntimeModule for DefinePropertyGettersRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/define_property_getters.js")).boxed())
  }
}
