use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

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

#[async_trait::async_trait]
impl RuntimeModule for AmdOptionsRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = {}",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::AMD_OPTIONS),
      self.options,
    ))
  }
}
