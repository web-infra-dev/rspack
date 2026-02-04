use rspack_collections::Identifier;
use rspack_core::{
  Compilation, OnPolicyCreationFailure, RuntimeGlobals, RuntimeModule, RuntimeTemplate,
  impl_runtime_module,
};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetTrustedTypesPolicyRuntimeModule {}

impl GetTrustedTypesPolicyRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetTrustedTypesPolicyRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_trusted_types_policy.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let trusted_types = compilation
      .options
      .output
      .trusted_types
      .as_ref()
      .expect("should have trusted_types");
    let runtime_requirements =
      get_chunk_runtime_requirements(compilation, &self.chunk.expect("should have chunk"));
    let wrap_policy_creation_in_try_catch = matches!(
      trusted_types.on_policy_creation_failure,
      OnPolicyCreationFailure::Continue
    );

    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_create_script": runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT),
        "_create_script_url": runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT_URL),
        "_wrap_try_catch": wrap_policy_creation_in_try_catch,
        "_policy_name": &trusted_types.policy_name.clone().unwrap_or_default(),
      })),
    )?;

    Ok(source)
  }
}
