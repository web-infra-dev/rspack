use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct SystemContextRuntimeModule {
  id: Identifier,
}

impl Default for SystemContextRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/start_entry_point"))
  }
}

impl RuntimeModule for SystemContextRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(format!(
        "{} = __system_context__",
        RuntimeGlobals::SYSTEM_CONTEXT
      ))
      .boxed(),
    )
  }
}
