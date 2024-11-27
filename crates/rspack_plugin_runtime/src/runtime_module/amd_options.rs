use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AmdOptionsRuntimeModule {
  id: Identifier,
  options: String,
}

impl AmdOptionsRuntimeModule {
  pub fn new(options: String) -> Self {
    Self::with_default(Identifier::from("webpack/runtime/amd_options"), options)
  }
}

impl RuntimeModule for AmdOptionsRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(format!(
        "{} = {}",
        RuntimeGlobals::AMD_OPTIONS.name(),
        self.options
      ))
      .boxed(),
    )
  }
}
