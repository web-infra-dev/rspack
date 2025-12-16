//! # EmbedFederationRuntimeModule
//!
//! Runtime module that wraps the startup function to ensure federation runtime dependencies
//! execute before other modules. Generates a "prevStartup wrapper" pattern with defensive
//! checks that intercepts and modifies the startup execution order.

use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, DependencyId, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  RuntimeTemplate, impl_runtime_module,
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
  pub fn new(
    runtime_template: &RuntimeTemplate,
    options: EmbedFederationRuntimeModuleOptions,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}embed_federation_runtime",
        runtime_template.runtime_module_prefix()
      )),
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

      // Expose a once-guarded startup hook on __webpack_require__ and keep the implementation
      // minimal (single assignment + fewer temp bindings).
      code.push_str(&format!("{async_startup} = (function() {{\n"));
      code.push_str("\tvar hasRun = false;\n");
      code.push_str("\tvar result;\n");
      code.push_str("\treturn function __webpack_require__mfAsyncStartup() {\n");
      code.push_str("\t\tif (hasRun) return result;\n");
      code.push_str("\t\thasRun = true;\n");
      // Run embedded federation runtime modules once (may return a Promise)
      code.push_str("\t\tvar base = (function(){\n");
      // Inline the federation runtime executions directly; keep newlines so the inserted code
      // remains syntactically valid inside the startup function body.
      code.push_str(&module_executions);
      code.push_str("\t\t})();\n");
      // Collect thenables from base + remote/consume handlers (mirrors webpack async startup)
      code
        .push_str("\t\tvar promises = base && typeof base.then === \"function\" ? [base] : [];\n");
      code.push_str("\t\tvar f = __webpack_require__.f;\n");
      code.push_str("\t\tif (f) {\n");
      code.push_str("\t\t\tvar startupChunkIds = [];\n");
      code.push_str("\t\t\tvar seen = {};\n");
      code.push_str("\t\t\tvar mapping;\n");
      code.push_str(
        "\t\t\tif (__webpack_require__.remotesLoadingData && (mapping = __webpack_require__.remotesLoadingData.chunkMapping)) {\n",
      );
      code.push_str("\t\t\t\tfor (var id in mapping) {\n");
      code
        .push_str("\t\t\t\t\tif (!Object.prototype.hasOwnProperty.call(mapping, id)) continue;\n");
      code.push_str("\t\t\t\t\tif (seen[id]) continue;\n");
      code.push_str("\t\t\t\t\tseen[id] = 1;\n");
      code.push_str("\t\t\t\t\tstartupChunkIds.push(id);\n");
      code.push_str("\t\t\t\t}\n");
      code.push_str("\t\t\t}\n");
      code.push_str(
        "\t\t\tif (__webpack_require__.consumesLoadingData && (mapping = __webpack_require__.consumesLoadingData.chunkMapping)) {\n",
      );
      code.push_str("\t\t\t\tfor (var id in mapping) {\n");
      code
        .push_str("\t\t\t\t\tif (!Object.prototype.hasOwnProperty.call(mapping, id)) continue;\n");
      code.push_str("\t\t\t\t\tif (seen[id]) continue;\n");
      code.push_str("\t\t\t\t\tseen[id] = 1;\n");
      code.push_str("\t\t\t\t\tstartupChunkIds.push(id);\n");
      code.push_str("\t\t\t\t}\n");
      code.push_str("\t\t\t}\n");
      code.push_str("\t\t\tif (startupChunkIds.length) {\n");
      code.push_str("\t\t\t\tvar consumes = f.consumes;\n");
      code.push_str("\t\t\t\tif (typeof consumes === \"function\") {\n");
      code.push_str(
        "\t\t\t\t\tfor (var i = 0; i < startupChunkIds.length; i++) consumes(startupChunkIds[i], promises);\n",
      );
      code.push_str("\t\t\t\t}\n");
      code.push_str("\t\t\t\tvar remotes = f.remotes;\n");
      code.push_str("\t\t\t\tif (typeof remotes === \"function\") {\n");
      code.push_str(
        "\t\t\t\t\tfor (var i = 0; i < startupChunkIds.length; i++) remotes(startupChunkIds[i], promises);\n",
      );
      code.push_str("\t\t\t\t}\n");
      code.push_str("\t\t\t}\n");
      code.push_str("\t\t}\n");
      code.push_str("\t\tresult = promises.length ? Promise.all(promises) : base;\n");
      code.push_str("\t\treturn result;\n");
      code.push_str("\t};\n");
      code.push_str("})();\n");

      code.push_str(&format!(
        "{startup} = (function(prev) {{\n\tvar fn = typeof prev === 'function' ? prev : function(){{}};\n\treturn function() {{\n\t\tvar res = {async_startup}();\n\t\tif (res && typeof res.then === \"function\") {{\n\t\t\treturn res.then(() => fn.apply(this, arguments));\n\t\t}}\n\t\treturn fn.apply(this, arguments);\n\t}};\n}})({startup});\n"
      ));

      // STARTUP_ENTRYPOINT is called with no args in runtimeChunk:'single' entry flows; tolerate that.
      code.push_str(&format!(
        "{startup_entry} = (function(prev) {{\n\tvar fn = typeof prev === 'function' ? prev : function(){{}};\n\treturn function(result, chunkIds, cb) {{\n\t\tvar res = {async_startup}();\n\t\tif (chunkIds === undefined && result === undefined) {{\n\t\t\treturn res && typeof res.then === \"function\" ? res.then(() => {{}}) : Promise.resolve();\n\t\t}}\n\t\tif (chunkIds === undefined) chunkIds = [];\n\t\tif (res && typeof res.then === \"function\") {{\n\t\t\treturn res.then(() => fn.call(this, result, chunkIds, cb));\n\t\t}}\n\t\treturn fn.call(this, result, chunkIds, cb);\n\t}};\n}})({startup_entry});\n"
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
}
