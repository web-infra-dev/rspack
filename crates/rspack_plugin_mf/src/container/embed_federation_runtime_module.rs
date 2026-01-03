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
      let require_global = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE);
      let entry_chunk_ids = compilation
        .chunk_by_ukey
        .expect_get(&chunk_ukey)
        .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
        .into_iter()
        .map(|chunk_ukey| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunk_ukey)
            .expect_id()
            .to_string()
        })
        .collect::<Vec<_>>();
      let entry_chunk_ids_literal =
        serde_json::to_string(&entry_chunk_ids).expect("Invalid json to string");

      let mut code = format!(
        r#"{require_global}.mfStartupBase = (function() {{
	var hasRun = false;
	var result;
	return function __webpack_require__mfStartupBase() {{
		if (hasRun) return result;
		hasRun = true;
		result = (function(){{
{module_executions}		}})();
		return result;
	}};
}})();

{async_startup} = (function() {{
	var hasRun = false;
	var result;
	return function __webpack_require__mfAsyncStartup() {{
		if (hasRun) return result;
		hasRun = true;
		var base = {require_global}.mfStartupBase && {require_global}.mfStartupBase();
		var promises = base && typeof base.then === "function" ? [base] : [];
		var f = __webpack_require__.f;
		if (f) {{
			var startupChunkIds = {entry_chunk_ids_literal};
			if (startupChunkIds.length) {{
				var consumes = f.consumes;
				if (typeof consumes === "function") {{
					for (var i = 0; i < startupChunkIds.length; i++) consumes(startupChunkIds[i], promises);
				}}
				var remotes = f.remotes;
				if (typeof remotes === "function") {{
					for (var i = 0; i < startupChunkIds.length; i++) remotes(startupChunkIds[i], promises);
				}}
			}}
		}}
		result = promises.length ? Promise.all(promises) : base;
		return result;
	}};
}})();

{require_global}.mfStartup = {startup};
"#,
        async_startup = async_startup,
        entry_chunk_ids_literal = entry_chunk_ids_literal,
        module_executions = module_executions,
        require_global = require_global,
        startup = startup,
      );

      let supports_arrow_function = compilation
        .options
        .output
        .environment
        .supports_arrow_function();
      if supports_arrow_function {
        code.push_str(&format!(
          r#"{startup} = (function(prev) {{
	var fn = typeof prev === 'function' ? prev : function(){{}};
	return function() {{
		var res = {async_startup}();
		if (res && typeof res.then === "function") {{
			return res.then(() => fn.apply(this, arguments));
		}}
		return fn.apply(this, arguments);
	}};
}})({startup});
"#,
          async_startup = async_startup,
          startup = startup,
        ));

        // STARTUP_ENTRYPOINT is called with no args in runtimeChunk:'single' entry flows; tolerate that.
        code.push_str(&format!(
          r#"{startup_entry} = (function(prev) {{
	var fn = typeof prev === 'function' ? prev : function(){{}};
	return function(result, chunkIds, cb) {{
		var res = {async_startup}();
		if (chunkIds === undefined && result === undefined) {{
			return res && typeof res.then === "function" ? res.then(() => {{}}) : Promise.resolve();
		}}
		if (chunkIds === undefined) chunkIds = [];
		if (res && typeof res.then === "function") {{
			return res.then(() => fn.call(this, result, chunkIds, cb));
		}}
		return fn.call(this, result, chunkIds, cb);
	}};
}})({startup_entry});
"#,
          async_startup = async_startup,
          startup_entry = startup_entry,
        ));
      } else {
        code.push_str(&format!(
          r#"{startup} = (function(prev) {{
	var fn = typeof prev === 'function' ? prev : function(){{}};
	return function() {{
		var res = {async_startup}();
		if (res && typeof res.then === "function") {{
			var _this = this, _args = arguments;
			return res.then(function() {{ return fn.apply(_this, _args); }});
		}}
		return fn.apply(this, arguments);
	}};
}})({startup});
"#,
          async_startup = async_startup,
          startup = startup,
        ));

        // STARTUP_ENTRYPOINT is called with no args in runtimeChunk:'single' entry flows; tolerate that.
        code.push_str(&format!(
          r#"{startup_entry} = (function(prev) {{
	var fn = typeof prev === 'function' ? prev : function(){{}};
	return function(result, chunkIds, cb) {{
		var res = {async_startup}();
		if (chunkIds === undefined && result === undefined) {{
			return res && typeof res.then === "function" ? res.then(function() {{}}) : Promise.resolve();
		}}
		if (chunkIds === undefined) chunkIds = [];
		if (res && typeof res.then === "function") {{
			var _this = this;
			return res.then(function() {{ return fn.call(_this, result, chunkIds, cb); }});
		}}
		return fn.call(this, result, chunkIds, cb);
	}};
}})({startup_entry});
"#,
          async_startup = async_startup,
          startup_entry = startup_entry,
        ));
      }

      Ok(code)
    } else {
      // Sync startup: keep the legacy prevStartup wrapper for minimal surface area.
      let startup = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::STARTUP);
      Ok(format!(
        r#"var prevStartup = {startup};
var hasRun = false;
{startup} = function() {{
	if (!hasRun) {{
		hasRun = true;
{module_executions}	}}
	if (typeof prevStartup === 'function') {{
		return prevStartup();
	}} else {{
		console.warn('[MF] Invalid prevStartup');
	}}
}};
"#,
        module_executions = module_executions,
        startup = startup,
      ))
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger // Run after RemoteRuntimeModule and StartupChunkDependenciesRuntimeModule
  }
}
