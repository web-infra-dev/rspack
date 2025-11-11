use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct SystemContextRuntimeModule {
  id: Identifier,
}

impl Default for SystemContextRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/start_entry_point"))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for SystemContextRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = __system_context__",
      RuntimeGlobals::API_SYSTEM_CONTEXT
    ))
  }
}
