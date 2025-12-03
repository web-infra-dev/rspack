use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct SystemContextRuntimeModule {
  id: Identifier,
}

impl SystemContextRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}system_context",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for SystemContextRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = __system_context__",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::SYSTEM_CONTEXT)
    ))
  }
}
