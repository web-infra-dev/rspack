use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct CreateScriptUrlRuntimeModule {
  id: Identifier,
}

impl Default for CreateScriptUrlRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/create_script_url"),
    }
  }
}

impl RuntimeModule for CreateScriptUrlRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    RawSource::from(format!(
      r#"
    {} = function(url){{
      return {};
    }};
    "#,
      RuntimeGlobals::CREATE_SCRIPT_URL,
      if compilation.options.output.trusted_types.is_some() {
        format!(
          "{}().createScriptURL(url)",
          RuntimeGlobals::GET_TRUSTED_TYPES_POLICY
        )
      } else {
        "'{url}'".to_string()
      }
    ))
    .boxed()
  }
}

impl_runtime_module!(CreateScriptUrlRuntimeModule);
