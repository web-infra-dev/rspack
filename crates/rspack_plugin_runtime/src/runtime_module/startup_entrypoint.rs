use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct StartupEntrypointRuntimeModule {
  id: Identifier,
  async_chunk_loading: bool,
}

impl StartupEntrypointRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, async_chunk_loading: bool) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}startup_entrypoint",
        runtime_template.runtime_module_prefix()
      )),
      async_chunk_loading,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StartupEntrypointRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      if self.async_chunk_loading {
        include_str!("runtime/startup_entrypoint_with_async.ejs").to_string()
      } else {
        include_str!("runtime/startup_entrypoint.ejs").to_string()
      },
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    compilation.runtime_template.render(&self.id, None)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    RuntimeGlobals::REQUIRE
      | RuntimeGlobals::ENSURE_CHUNK
      | RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES
  }
}
