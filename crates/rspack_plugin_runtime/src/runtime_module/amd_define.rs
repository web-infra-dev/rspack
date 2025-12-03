use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

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

#[async_trait::async_trait]
impl RuntimeModule for AmdDefineRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function () {{ throw new Error('define cannot be used indirect'); }}",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::AMD_DEFINE),
    ))
  }
}
