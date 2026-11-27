use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

use crate::extract_runtime_module_variables_from_ejs;

static STARTUP_CHUNK_DEPENDENCIES_TEMPLATE: &str =
  include_str!("runtime/startup_chunk_dependencies.ejs");
static RUNTIME_MODULE_VARIABLES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
  extract_runtime_module_variables_from_ejs(&[STARTUP_CHUNK_DEPENDENCIES_TEMPLATE])
});

#[impl_runtime_module]
#[derive(Debug)]
pub struct StartupChunkDependenciesRuntimeModule {
  async_chunk_loading: bool,
}

impl StartupChunkDependenciesRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, async_chunk_loading: bool) -> Self {
    Self::with_default(runtime_template, async_chunk_loading)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for StartupChunkDependenciesRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES.as_slice()
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let mut dependencies = RuntimeGlobals::STARTUP
      | RuntimeGlobals::ENSURE_CHUNK
      | RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES;
    if self.async_chunk_loading {
      dependencies.insert(RuntimeGlobals::REQUIRE);
    }
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      define: { RuntimeGlobals::STARTUP },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      STARTUP_CHUNK_DEPENDENCIES_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let graph = &context.compilation.build_chunk_graph_artifact;
    let chunk_ukey = self
      .chunk()
      .expect("should have chunk for StartupChunkDependenciesRuntimeModule");
    let chunk_ids = graph
      .chunk_graph
      .get_chunk_entry_dependent_chunks_iterable(
        &chunk_ukey,
        &graph.chunk_by_ukey,
        &graph.chunk_group_by_ukey,
      )
      .map(|chunk_ukey| graph.chunk_by_ukey.expect_get(&chunk_ukey).expect_id())
      .collect::<Vec<_>>();

    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_async_chunk_loading": self.async_chunk_loading,
        "_chunk_ids": chunk_ids,
      })),
    )
  }
}
