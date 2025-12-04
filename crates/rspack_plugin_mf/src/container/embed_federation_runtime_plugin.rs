//! # EmbedFederationRuntimePlugin
//!
//! Plugin that manages federation runtime initialization by adding STARTUP requirements
//! to federation-enabled chunks and injecting EmbedFederationRuntimeModule into runtime chunks.
//! Also handles explicit startup calls for entry chunks that delegate to runtime chunks.

use std::sync::{Arc, Mutex};

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, DependencyId, ModuleIdentifier, Plugin,
  RuntimeGlobals,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, JsPlugin, RenderSource};
use rustc_hash::FxHashSet;

use super::{
  embed_federation_runtime_module::{
    EmbedFederationRuntimeModule, EmbedFederationRuntimeModuleOptions,
  },
  federation_modules_plugin::{AddFederationRuntimeDependencyHook, FederationModulesPlugin},
  federation_runtime_dependency::FederationRuntimeDependency,
  module_federation_runtime_plugin::ModuleFederationRuntimeExperimentsOptions,
};

struct FederationRuntimeDependencyCollector {
  collected_dependency_ids: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait::async_trait]
impl AddFederationRuntimeDependencyHook for FederationRuntimeDependencyCollector {
  async fn run(&self, dependency: &FederationRuntimeDependency) -> Result<()> {
    self
      .collected_dependency_ids
      .lock()
      .expect("Failed to lock collected_dependency_ids")
      .insert(dependency.id);
    Ok(())
  }
}

#[plugin]
#[derive(Debug)]
pub struct EmbedFederationRuntimePlugin {
  experiments: ModuleFederationRuntimeExperimentsOptions,
  collected_dependency_ids: Arc<Mutex<FxHashSet<DependencyId>>>,
}

impl EmbedFederationRuntimePlugin {
  pub fn new(experiments: ModuleFederationRuntimeExperimentsOptions) -> Self {
    Self::new_inner(experiments, Arc::new(Mutex::new(FxHashSet::default())))
  }
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_chunk_runtime_requirements_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Check if chunk needs federation runtime support
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Federation is enabled for runtime chunks or entry chunks
  let is_enabled = has_runtime || has_entry_modules;

  if is_enabled {
    // Add STARTUP (sync) so the runtime wrapper can always hook __webpack_require__.x,
    // and STARTUP_ENTRYPOINT (async) when async startup is enabled.
    if self.experiments.async_startup {
      runtime_requirements.insert(RuntimeGlobals::STARTUP);
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::ASYNC_FEDERATION_STARTUP);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    } else {
      runtime_requirements.insert(RuntimeGlobals::STARTUP);
    }
  }

  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for EmbedFederationRuntimePlugin)]
async fn runtime_requirement_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  _runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(None);
  }

  // Only inject EmbedFederationRuntimeModule into runtime chunks
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  if has_runtime {
    // Collect federation dependencies snapshot
    let collected_ids_snapshot = self
      .collected_dependency_ids
      .lock()
      .expect("Failed to lock collected_dependency_ids")
      .iter()
      .cloned()
      .collect::<Vec<DependencyId>>();

    let emro = EmbedFederationRuntimeModuleOptions {
      collected_dependency_ids: collected_ids_snapshot,
      experiments: self.experiments.clone(),
    };

    // Inject EmbedFederationRuntimeModule
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(EmbedFederationRuntimeModule::new(emro)),
    )?;
  }

  Ok(None)
}

#[plugin_hook(CompilerCompilation for EmbedFederationRuntimePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let collector = FederationRuntimeDependencyCollector {
    collected_dependency_ids: Arc::clone(&self.collected_dependency_ids),
  };

  let federation_hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

  federation_hooks
    .add_federation_runtime_dependency
    .lock()
    .await
    .tap(collector);

  // Register render startup hook, patches entrypoints
  let js_hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  js_hooks
    .write()
    .await
    .render_startup
    .tap(render_startup::new(self));

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for EmbedFederationRuntimePlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return Ok(());
  }

  // Only process chunks that have federation dependencies
  let collected_deps = self
    .collected_dependency_ids
    .lock()
    .expect("Failed to lock collected_dependency_ids")
    .iter()
    .cloned()
    .collect::<Vec<DependencyId>>();
  let has_federation_deps = !collected_deps.is_empty();

  if !has_federation_deps {
    return Ok(());
  }

  let async_startup_global = compilation
    .runtime_template
    .render_runtime_globals(&RuntimeGlobals::ASYNC_FEDERATION_STARTUP);

  // Ensure the synchronous startup global exists even in async startup mode so the
  // embedded federation runtime wrapper has a stable hook point.
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Runtime chunks with entry modules
  if has_runtime && has_entry_modules {
    // For async startup, make sure the federation runtime is installed before
    // any chunk ensure handlers run. We can't change the templates here, so
    // prepend a guard that triggers the embedded federation startup once,
    // collects ensure-handler promises for all known federation chunk ids,
    // and returns a Promise so the caller can await remote loading before the
    // entry executes (mirrors webpack async startup behaviour).
    if self.experiments.async_startup {
      let mut with_startup = ConcatSource::default();
      with_startup.add(RawStringSource::from(format!(
        r#"(function() {{
  var originalMfAsyncStartup = typeof {async_startup_global} === "function" ? {async_startup_global} : null;
  {async_startup_global} = function() {{
    var base = originalMfAsyncStartup && originalMfAsyncStartup();
    var promises = base && typeof base.then === "function" ? [base] : [];

    var startupChunkIds = [];
    if (__webpack_require__.remotesLoadingData && __webpack_require__.remotesLoadingData.chunkMapping) {{
      startupChunkIds.push.apply(startupChunkIds, Object.keys(__webpack_require__.remotesLoadingData.chunkMapping));
    }}
    if (__webpack_require__.consumesLoadingData && __webpack_require__.consumesLoadingData.chunkMapping) {{
      startupChunkIds.push.apply(startupChunkIds, Object.keys(__webpack_require__.consumesLoadingData.chunkMapping));
    }}
    if (!startupChunkIds.length) return base;

    var f = __webpack_require__.f || {{}};
    var handlers = [f.consumes, f.remotes];
    for (var i = 0; i < handlers.length; i++) {{
      var h = handlers[i];
      if (typeof h !== "function") continue;
      for (var j = 0; j < startupChunkIds.length; j++) {{
        h(startupChunkIds[j], promises);
      }}
    }}

    if (promises.length) return Promise.all(promises);
    return base;
  }};
}})();
"#,
      )));
      with_startup.add(render_source.source.clone());
      render_source.source = with_startup.boxed();
    }
    return Ok(());
  }

  // Entry chunks delegating to runtime need explicit startup calls
  if !has_runtime && has_entry_modules {
    let mut startup_with_call = ConcatSource::default();

    // Add startup call
    startup_with_call.add(RawStringSource::from_static(
      "\n// Federation startup call\n",
    ));
    let startup_global = if self.experiments.async_startup {
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::STARTUP_ENTRYPOINT)
    } else {
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::STARTUP)
    };
    startup_with_call.add(RawStringSource::from(format!("{startup_global}();\n",)));

    startup_with_call.add(render_source.source.clone());
    render_source.source = startup_with_call.boxed();
  }

  Ok(())
}

impl Plugin for EmbedFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "EmbedFederationRuntimePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements_tree::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirement_in_tree::new(self));
    Ok(())
  }
}

impl Default for EmbedFederationRuntimePlugin {
  fn default() -> Self {
    Self::new(ModuleFederationRuntimeExperimentsOptions::default())
  }
}
