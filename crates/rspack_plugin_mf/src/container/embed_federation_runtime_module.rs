//! # EmbedFederationRuntimeModule
//!
//! Runtime module that wraps the startup function to ensure federation runtime dependencies
//! execute before other modules. Generates a "prevStartup wrapper" pattern with defensive
//! checks that intercepts and modifies the startup execution order.

use cow_utils::CowUtils;
use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, DependencyId, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  impl_runtime_module,
};
use rspack_error::Result;

use super::module_federation_runtime_plugin::ModuleFederationRuntimeExperimentsOptions;

#[cacheable]
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EmbedFederationRuntimeModuleOptions {
  pub collected_dependency_ids: Vec<DependencyId>,
  pub experiments: ModuleFederationRuntimeExperimentsOptions,
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
    let mut module_executions = String::with_capacity(federation_runtime_modules.len() * 64);

    for dep_id in federation_runtime_modules {
      let module_str = compilation.runtime_template.module_raw(
        compilation,
        &mut runtime_requirements,
        &dep_id,
        "",
        false,
      );
      module_executions.push_str("\t\t");
      module_executions.push_str(&module_str);
      module_executions.push('\n');
    }

    if self.options.experiments.async_startup {
      // Build startup wrappers. Always wrap STARTUP; also wrap STARTUP_ENTRYPOINT when async startup
      // is enabled so runtimeChunk: "single" flows still initialize federation runtime.
      let startup = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::STARTUP);
      let startup_entry = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::STARTUP_ENTRYPOINT);
      let async_startup = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ASYNC_FEDERATION_STARTUP);

      let mut code = String::with_capacity(256 + module_executions.len());

      // Expose once-guarded startup on __webpack_require__ so other runtime pieces can call it
      code.push_str("var __webpack_require__mf_has_run = false;\n");
      code.push_str("var __webpack_require__mf_startup_result;\n");
      code.push_str("function __webpack_require__mfAsyncStartup() {\n");
      code.push_str(
        "\tif (__webpack_require__mf_has_run) return __webpack_require__mf_startup_result;\n",
      );
      code.push_str("\t__webpack_require__mf_has_run = true;\n");
      code.push_str("\t__webpack_require__mf_startup_result = (function(){\n");
      let module_exec_escaped = module_executions.cow_replace("\n", "\\n");
      code.push_str(module_exec_escaped.as_ref());
      code.push_str("\t})();\n");
      code.push_str("\treturn __webpack_require__mf_startup_result;\n");
      code.push_str("}\n");
      code.push_str(&format!(
        "{async_startup} = __webpack_require__mfAsyncStartup;\n"
      ));

      code.push_str("function __webpack_require__mf_wrapStartup(prev) {\n");
      code.push_str("\tvar fn = typeof prev === 'function' ? prev : function(){};\n");
      code.push_str("\treturn function() {\n");
      code.push_str("\t\tvar res = __webpack_require__mfAsyncStartup();\n");
      code.push_str("\t\tif (res && typeof res.then === \"function\") {\n");
      code.push_str("\t\t\treturn res.then(() => fn.apply(this, arguments));\n");
      code.push_str("\t\t}\n");
      code.push_str("\t\treturn fn.apply(this, arguments);\n");
      code.push_str("\t};\n");
      code.push_str("}\n");

      // Make STARTUP_ENTRYPOINT tolerant to zero-arg calls (runtimeChunk: 'single' startup path)
      code.push_str("var __webpack_require__startup = __webpack_require__.X;\n");
      code.push_str("function __webpack_require__startup_guard(result, chunkIds, fn) {\n");
      code.push_str(
        "\tif (chunkIds === undefined && result === undefined) return Promise.resolve();\n",
      );
      code.push_str("\tif (chunkIds === undefined) chunkIds = [];\n");
      code.push_str("\treturn __webpack_require__startup.call(this, result, chunkIds, fn);\n");
      code.push_str("}\n");
      code.push_str("__webpack_require__.X = __webpack_require__startup_guard;\n");

      code.push_str(&format!(
        "{startup} = __webpack_require__mf_wrapStartup({startup});\n"
      ));
      code.push_str(&format!(
        "{startup_entry} = __webpack_require__mf_wrapStartup({startup_entry});\n"
      ));

      Ok(code)
    } else {
      // Sync startup: keep the legacy prevStartup wrapper for minimal surface area.
      let startup = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::STARTUP);
      let mut code = String::with_capacity(128 + module_executions.len());
      code.push_str(&format!("var prevStartup = {startup};\n"));
      code.push_str("var hasRun = false;\n");
      code.push_str(&format!("{startup} = function() {{\n"));
      code.push_str("\tif (!hasRun) {\n");
      code.push_str("\t\thasRun = true;\n");
      code.push_str(&module_executions);
      code.push_str("\t}\n");
      code.push_str("\tif (typeof prevStartup === 'function') {\n");
      code.push_str("\t\treturn prevStartup();\n");
      code.push_str("\t} else {\n");
      code.push_str("\t\tconsole.warn('[MF] Invalid prevStartup');\n");
      code.push_str("\t}\n");
      code.push_str("};\n");
      Ok(code)
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger // Run after RemoteRuntimeModule and StartupChunkDependenciesRuntimeModule
  }

  fn should_isolate(&self) -> bool {
    true
  }
}
