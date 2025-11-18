use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, OnPolicyCreationFailure, RuntimeGlobals, RuntimeModule,
  impl_runtime_module,
};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetTrustedTypesPolicyRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for GetTrustedTypesPolicyRuntimeModule {
  fn default() -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/get_trusted_types_policy"),
      None,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetTrustedTypesPolicyRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

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

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
