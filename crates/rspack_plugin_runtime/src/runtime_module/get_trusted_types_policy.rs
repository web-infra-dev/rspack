use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, OnPolicyCreationFailure, RuntimeGlobals, RuntimeModule,
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
    let wrap_policy_creation_in_try_catch = matches!(
      trusted_types.on_policy_creation_failure,
      OnPolicyCreationFailure::Continue
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
    let wrap_policy_creation_try_catch_start = if wrap_policy_creation_in_try_catch {
      "try {"
    } else {
      ""
    };
    let wrap_policy_creation_try_catch_end = if wrap_policy_creation_in_try_catch {
      format!(
        r#"
          }} catch (e) {{
            console.warn('Could not create trusted-types policy {}');
          }}
        "#,
        serde_json::to_string(&trusted_types.policy_name.clone().unwrap_or_default())
          .expect("invalid json to_string"),
      )
    } else {
      "".to_string()
    };

    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_policy_content": policy_content.join(",\n"),
        "_wrap_policy_creation_try_catch_start": wrap_policy_creation_try_catch_start,
        "_warp_policy_creation_try_catch_end": wrap_policy_creation_try_catch_end,
        "_policy_name": &trusted_types.policy_name.clone().unwrap_or_default(),
      })),
    )?;

    Ok(RawStringSource::from(source).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
