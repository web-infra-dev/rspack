use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct NonceRuntimeModule {
  id: Identifier,
}

impl Default for NonceRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/nonce"))
  }
}

impl RuntimeModule for NonceRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawStringSource::from(format!("{} = undefined;", RuntimeGlobals::SCRIPT_NONCE)).boxed())
  }
}
