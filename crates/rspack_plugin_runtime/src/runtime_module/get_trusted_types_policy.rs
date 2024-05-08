use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct GetTrustedTypesPolicyRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for GetTrustedTypesPolicyRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/get_trusted_types_policy"),
      chunk: None,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for GetTrustedTypesPolicyRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let trusted_types = compilation
      .options
      .output
      .trusted_types
      .as_ref()
      .expect("should have trusted_types");
    let runtime_requirements =
      get_chunk_runtime_requirements(compilation, &self.chunk.expect("should have chunk"));
    let create_script = runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT);
    let create_script_url = runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT_URL);

    let mut result = include_str!("runtime/get_trusted_types_policy.js").replace(
      "$policyName$",
      &trusted_types.policy_name.clone().unwrap_or_default(),
    );
    let mut policy_content: Vec<String> = Vec::new();
    if create_script {
      policy_content.push(
        r#"
        createScript: function (script) {
          return script;
        }
        "#
        .to_string(),
      );
    }
    if create_script_url {
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
    Ok(RawSource::from(result).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
