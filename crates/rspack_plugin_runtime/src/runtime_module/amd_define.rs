use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AmdDefineRuntimeModule {
  id: Identifier,
}

impl Default for AmdDefineRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/amd_define"))
  }
}

impl RuntimeModule for AmdDefineRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawStringSource::from(format!(
        "{} = function () {{ throw new Error('define cannot be used indirect'); }}",
        RuntimeGlobals::AMD_DEFINE.name()
      ))
      .boxed(),
    )
  }
}
