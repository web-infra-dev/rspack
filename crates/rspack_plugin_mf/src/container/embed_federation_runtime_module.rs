//! # EmbedFederationRuntimeModule
//!
//! Runtime module that wraps the startup function to ensure federation runtime dependencies
//! execute before other modules. Generates a "prevStartup wrapper" pattern with defensive
//! checks that intercepts and modifies the startup execution order.

use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, DependencyId, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  impl_runtime_module, module_raw,
};
use rspack_error::Result;

#[cacheable]
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EmbedFederationRuntimeModuleOptions {
  pub collected_dependency_ids: Vec<DependencyId>,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct EmbedFederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  options: EmbedFederationRuntimeModuleOptions,
}

impl EmbedFederationRuntimeModule {
  pub fn new(options: EmbedFederationRuntimeModuleOptions) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/embed_federation_runtime"),
      None,
      options,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EmbedFederationRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    let chunk_ukey = self
      .chunk
      .expect("Chunk should be attached to RuntimeModule");

    let collected_deps = &self.options.collected_dependency_ids;

    if collected_deps.is_empty() {
      return Ok("// No federation runtime dependencies to embed.".into());
    }

    let module_graph = compilation.get_module_graph();
    let mut federation_runtime_modules = Vec::new();

    // Find federation runtime dependencies in this chunk
    for dep_id in collected_deps.iter() {
      if let Some(module_dyn) = module_graph.get_module_by_dependency_id(dep_id) {
        let is_in_chunk = compilation
          .chunk_graph
          .is_module_in_chunk(&module_dyn.identifier(), chunk_ukey);
        if is_in_chunk {
          federation_runtime_modules.push(*dep_id);
        }
      }
    }

    if federation_runtime_modules.is_empty() {
      return Ok("// Federation runtime entry modules not found in this chunk.".into());
    }

    // Generate module execution code for each federation runtime dependency
    let mut runtime_requirements = RuntimeGlobals::default();
    let mut module_executions = String::with_capacity(federation_runtime_modules.len() * 50);

    for dep_id in federation_runtime_modules {
      let module_str = module_raw(compilation, &mut runtime_requirements, &dep_id, "", false);
      module_executions.push_str("\t\t");
      module_executions.push_str(&module_str);
      module_executions.push('\n');
    }
    // Remove trailing newline
    if !module_executions.is_empty() {
      module_executions.pop();
    }

    // Generate prevStartup wrapper pattern with defensive checks
    // When mf_async_startup is enabled, wrap __webpack_require__.x instead of STARTUP_ENTRYPOINT
    // This ensures federation runtime modules execute BEFORE __webpack_require__.x() is called
    let result = if compilation.options.experiments.mf_async_startup {
      format!(
        r#"var prevX = __webpack_require__.x;
var hasRun = false;
__webpack_require__.x = function() {{
	if (!hasRun) {{
		hasRun = true;
{module_executions}
	}}
	if (typeof prevX === 'function') {{
		return prevX.apply(this, arguments);
	}}
}};"#
      )
    } else {
      let startup = RuntimeGlobals::STARTUP.name();
      format!(
        r#"var prevStartup = {startup};
var hasRun = false;
{startup} = function() {{
	if (!hasRun) {{
		hasRun = true;
{module_executions}
	}}
	if (typeof prevStartup === 'function') {{
		return prevStartup();
	}} else {{
		console.warn('[MF] Invalid prevStartup');
	}}
}};"#
      )
    };

    Ok(result)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger // Run after RemoteRuntimeModule and StartupChunkDependenciesRuntimeModule
  }
}
