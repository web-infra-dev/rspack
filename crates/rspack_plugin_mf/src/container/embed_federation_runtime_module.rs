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
  pub async_startup: bool,
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

    let module_exec_body = if module_executions.is_empty() {
      "\t// no federation runtime modules to execute\n".to_string()
    } else {
      format!("{module_executions}\n")
    };

    let run_function = format!(
      "function runFederationRuntime() {{\n\tif (hasRun) return;\n\thasRun = true;\n{module_exec_body}}}\n",
      module_exec_body = module_exec_body
    );

    let install_chunk = RuntimeGlobals::EXTERNAL_INSTALL_CHUNK.name();

    // Generate wrapper pattern ensuring federation runtime executes before startup or chunk installation runs
    let mut result = String::new();
    if self.options.async_startup {
      let startup = RuntimeGlobals::STARTUP_ENTRYPOINT.name();
      result.push_str(&format!("var prevStartup = {};\n", startup));
      result.push_str("var hasRun = false;\n");
      result.push_str("var __mfInitialConsumes;\n");
      result.push_str(
        "function __mfSetupInitialConsumesDeferral() {\n\tvar runtime = __webpack_require__.federation && __webpack_require__.federation.bundlerRuntime;\n\tif (!runtime || typeof runtime.installInitialConsumes !== 'function') return;\n\tvar origInstallInitialConsumes = runtime.installInitialConsumes;\n\truntime.installInitialConsumes = function(opts) { __mfInitialConsumes = opts; };\n\truntime.flushInitialConsumes = function() { if (__mfInitialConsumes) { var opts = __mfInitialConsumes; __mfInitialConsumes = undefined; return origInstallInitialConsumes(opts); } };\n}\n",
      );
      result.push_str(&run_function);
      result.push_str(&format!(
        "{startup} = function() {{\n\trunFederationRuntime();\n\t__mfSetupInitialConsumesDeferral();\n\tif (typeof prevStartup === 'function') {{\n\t\treturn prevStartup.apply(this, arguments);\n\t}} else {{\n\t\tconsole.warn('[MF] Invalid prevStartup');\n\t}}\n}};\n",
        startup = startup
      ));
    } else {
      let startup = RuntimeGlobals::STARTUP.name();
      result.push_str(&format!("var prevStartup = {startup};\n"));
      result.push_str("var hasRun = false;\n");
      result.push_str(&run_function);
      result.push_str(&format!(
        "{startup} = function() {{\n\trunFederationRuntime();\n\tif (typeof prevStartup === 'function') {{\n\t\treturn prevStartup.apply(this, arguments);\n\t}} else {{\n\t\tconsole.warn('[MF] Invalid prevStartup');\n\t}}\n}};\n",
        startup = startup
      ));
    }

    result.push_str(&format!(
      "var prevExternalInstallChunk = {install_chunk};\nif (typeof prevExternalInstallChunk === 'function') {{\n\t{install_chunk} = function() {{\n\t\trunFederationRuntime();\n\t\treturn prevExternalInstallChunk.apply(this, arguments);\n\t}};\n}}\n",
      install_chunk = install_chunk
    ));

    // For async startup we defer executing the federation runtime until the
    // async startup entrypoint runs (or a chunk install occurs) so that share
    // scope initialization can complete first. In sync mode we keep the eager
    // execution to match the legacy bootstrap order.
    if !self.options.async_startup {
      result.push_str("runFederationRuntime();\n");
    }

    Ok(result)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger // Run after RemoteRuntimeModule and StartupChunkDependenciesRuntimeModule
  }
}
