use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeGlobals, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateScriptRuntimeModule {
  id: Identifier,
}

impl Default for CreateScriptRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/create_script"))
  }
}

impl RuntimeModule for CreateScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      r#"
    {} = function(script){{
      return {};
    }};
    "#,
      RuntimeGlobals::CREATE_SCRIPT,
      if compilation.options.output.trusted_types.is_some() {
        format!(
          "{}().createScript(script)",
          RuntimeGlobals::GET_TRUSTED_TYPES_POLICY
        )
      } else {
        "script".to_string()
      }
    ))
  }
}
