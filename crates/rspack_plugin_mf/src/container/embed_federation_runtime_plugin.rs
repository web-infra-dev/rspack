//! # EmbedFederationRuntimePlugin
//!
//! Plugin that manages federation runtime initialization by adding STARTUP requirements
//! to federation-enabled chunks and injecting EmbedFederationRuntimeModule into runtime chunks.
//! Also handles explicit startup calls for entry chunks that delegate to runtime chunks.

use std::sync::{Arc, Mutex};

use rspack_core::{
  ChunkGraph, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAdditionalTreeRuntimeRequirements, CompilationParams,
  CompilationRuntimeRequirementInTree, CompilerCompilation, DependencyId, ModuleIdentifier, Plugin,
  RuntimeGlobals,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, JsPlugin, RenderSource};
use rustc_hash::FxHashSet;

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_module::{
    EmbedFederationRuntimeModule, EmbedFederationRuntimeModuleOptions,
  },
  federation_modules_plugin::{AddFederationRuntimeDependencyHook, FederationModulesPlugin},
  federation_runtime_dependency::FederationRuntimeDependency,
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
  async_startup: bool,
  collected_dependency_ids: Arc<Mutex<FxHashSet<DependencyId>>>,
}

impl EmbedFederationRuntimePlugin {
  pub fn new(async_startup: bool) -> Self {
    Self::new_inner(async_startup, Arc::new(Mutex::new(FxHashSet::default())))
  }

  /// Check if the chunk is a container entry chunk (should NOT use async startup)
  fn is_container_entry_chunk(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
    let entries = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);

    // Inspect the last entry module (final entry) and its chunk group metadata
    if let Some((module_id, chunk_group_ukey)) = entries.iter().next_back() {
      let module_graph = compilation.get_module_graph();
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        return module.as_any().is::<ContainerEntryModule>();
      }
      let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group_ukey);
      if let Some(entry_options) = chunk_group.kind.get_entry_options()
        && entry_options.library.is_some()
      {
        return true;
      }
    }
    false
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
  let is_container_entry_chunk = Self::is_container_entry_chunk(compilation, chunk_ukey);
  let use_async_startup = self.async_startup && !is_container_entry_chunk;

  if is_enabled {
    // Add STARTUP or STARTUP_ENTRYPOINT based on mf_async_startup experiment
    if use_async_startup {
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
      runtime_requirements.insert(RuntimeGlobals::ASYNC_FEDERATION_STARTUP);
    } else {
      runtime_requirements.insert(RuntimeGlobals::STARTUP);
    }
  }

  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
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
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Only add requirements for entry chunks (non-runtime chunks with entries)
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  if has_runtime || !has_entry_modules {
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

  let is_container_entry_chunk = Self::is_container_entry_chunk(compilation, chunk_ukey);
  let use_async_startup = self.async_startup && !is_container_entry_chunk;

  if use_async_startup {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    runtime_requirements.insert(RuntimeGlobals::ASYNC_FEDERATION_STARTUP);
  } else {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
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
      async_startup: self.async_startup,
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

  // Register render startup hook to patch entrypoints when needed
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

  if self.async_startup && Self::is_container_entry_chunk(compilation, chunk_ukey) {
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

  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Runtime chunks with entry modules: JavaScript plugin handles startup naturally
  if has_runtime && has_entry_modules {
    return Ok(());
  }

  // Entry chunks delegating to runtime need explicit startup calls
  if !has_runtime && has_entry_modules {
    let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
    let mut startup_with_call = ConcatSource::default();

    // Add startup call
    startup_with_call.add(RawStringSource::from_static(
      "\n// Federation startup call\n",
    ));
    let startup_global = if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
      RuntimeGlobals::STARTUP_ENTRYPOINT.name()
    } else {
      RuntimeGlobals::STARTUP.name()
    };
    startup_with_call.add(RawStringSource::from(format!("{startup_global}();\n")));

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
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirement_in_tree::new(self));
    Ok(())
  }
}

impl Default for EmbedFederationRuntimePlugin {
  fn default() -> Self {
    Self::new(false)
  }
}
