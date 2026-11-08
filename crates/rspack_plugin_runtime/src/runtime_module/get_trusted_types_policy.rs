use std::sync::LazyLock;

use rspack_core::{
  Compilation, OnPolicyCreationFailure, RuntimeGlobals, RuntimeModule,
  RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

use crate::{extract_runtime_module_variables_from_ejs, get_chunk_runtime_requirements};

static GET_TRUSTED_TYPES_POLICY_TEMPLATE: &str =
  include_str!("runtime/get_trusted_types_policy.ejs");
static RUNTIME_MODULE_VARIABLES: LazyLock<Vec<&'static str>> =
  LazyLock::new(|| extract_runtime_module_variables_from_ejs(&[GET_TRUSTED_TYPES_POLICY_TEMPLATE]));

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
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES.as_slice()
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::GET_TRUSTED_TYPES_POLICY },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      GET_TRUSTED_TYPES_POLICY_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let trusted_types = compilation
      .options
      .output
      .trusted_types
      .as_ref()
      .expect("should have trusted_types");
    let runtime_requirements =
      get_chunk_runtime_requirements(compilation, &self.chunk().expect("should have chunk"));
    let wrap_policy_creation_in_try_catch = matches!(
      trusted_types.on_policy_creation_failure,
      OnPolicyCreationFailure::Continue
    );

    let source = context.runtime_template.render(
      self.id(),
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
