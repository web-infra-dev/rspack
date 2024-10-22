use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateScriptUrlRuntimeModule {
  id: Identifier,
}

impl Default for CreateScriptUrlRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/create_script_url"))
  }
}

impl RuntimeModule for CreateScriptUrlRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let generated_code = format!(
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
        "url".to_string()
      }
    );

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}
