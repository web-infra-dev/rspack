use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

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

#[async_trait::async_trait]
impl RuntimeModule for NonceRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = undefined;",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::SCRIPT_NONCE)
    ))
  }
}
