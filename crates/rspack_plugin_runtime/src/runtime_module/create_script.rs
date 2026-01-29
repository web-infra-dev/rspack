use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateScriptRuntimeModule {
  id: Identifier,
}

impl CreateScriptRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}create_script",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CreateScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/create_script.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_trusted_types": compilation.options.output.trusted_types.is_some(),
      })),
    )?;

    Ok(source)
  }

  fn additional_runtime_requirements(
    &self,
    compilation: &Compilation,
    _runtime_requirements: &RuntimeGlobals,
  ) -> RuntimeGlobals {
    if compilation.options.output.trusted_types.is_some() {
      RuntimeGlobals::GET_TRUSTED_TYPES_POLICY
    } else {
      RuntimeGlobals::default()
    }
  }
}
