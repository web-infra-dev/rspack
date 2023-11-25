use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

#[derive(Debug, Eq)]
pub struct GetTrustedTypesPolicyRuntimeModule {
  id: Identifier,
  create_script: bool,
  create_script_url: bool,
}

impl GetTrustedTypesPolicyRuntimeModule {
  pub fn new(runtime_requirements: &RuntimeGlobals) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/get_trusted_types_policy"),
      create_script: runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT),
      create_script_url: runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT_URL),
    }
  }
}

impl RuntimeModule for GetTrustedTypesPolicyRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let trusted_types = compilation
      .options
      .output
      .trusted_types
      .as_ref()
      .expect("should have trusted_types");

    let mut result = include_str!("runtime/get_trusted_types_policy.js").replace(
      "$policyName$",
      &trusted_types.policy_name.clone().unwrap_or_default(),
    );
    let mut policy_content: Vec<String> = Vec::new();
    if self.create_script {
      policy_content.push(
        r#"
        createScript: function (script) {
          return script;
        }
        "#
        .to_string(),
      );
    }
    if self.create_script_url {
      policy_content.push(
        r#"
        createScriptURL: function (url) {
          return url;
        }
        "#
        .to_string(),
      );
    }
    result = result.replace("$policyContent$", policy_content.join(",\n").as_ref());
    RawSource::from(result).boxed()
  }
}

impl_runtime_module!(GetTrustedTypesPolicyRuntimeModule);
