use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};

use rspack_core::{
  ApplyContext, ChunkGraph, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions,
  DependencyId, ModuleIdentifier, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesRenderStartup, JsPlugin, RenderSource};
use rspack_sources::{ConcatSource, RawStringSource, SourceExt};

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_module::{
    EmbedFederationRuntimeModule, EmbedFederationRuntimeModuleOptions,
  },
  federation_modules_plugin::{AddFederationRuntimeDependencyHook, FederationModulesPlugin},
  federation_runtime_dependency::FederationRuntimeDependency,
};

#[derive(Debug, Default)]
struct EmbedFederationRuntimePluginOptions {
  // Currently no options, but can be extended later
}

struct FederationRuntimeDependencyCollector {
  collected_dependency_ids: Arc<Mutex<HashSet<DependencyId>>>,
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
  #[allow(dead_code)]
  options: EmbedFederationRuntimePluginOptions,
  collected_dependency_ids: Arc<Mutex<HashSet<DependencyId>>>,
}

impl EmbedFederationRuntimePlugin {
  pub fn new() -> Self {
    Self::new_inner(
      EmbedFederationRuntimePluginOptions::default(),
      Arc::new(Mutex::new(HashSet::new())),
    )
  }
}

// Helper to check if a chunk should get the federation runtime module
fn is_enabled_for_chunk(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Skip build time chunks
  if chunk.name() == Some("build time chunk") {
    return false;
  }

  // Get entry modules for the chunk
  let entry_modules = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);

  // Check if this is a container chunk (has ContainerEntryModule)
  if let Some(module_id) = entry_modules.last() {
    let module_graph = compilation.get_module_graph();
    if let Some(module) = module_graph.module_by_identifier(module_id) {
      if module.as_any().is::<ContainerEntryModule>() {
        // Container chunks NEED the runtime module for initContainer/getContainer functions
        return true;
      }
    }
  }

  // Regular entry chunks also need the runtime module
  true
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_chunk_runtime_requirements_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
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
  if is_enabled_for_chunk(compilation, chunk_ukey) {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      let collected_ids_snapshot = self
        .collected_dependency_ids
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<DependencyId>>();
      let emro = EmbedFederationRuntimeModuleOptions {
        collected_dependency_ids: collected_ids_snapshot,
      };
      compilation.add_runtime_module(
        chunk_ukey,
        Box::new(EmbedFederationRuntimeModule::new(emro)),
      )?;
    }
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

  // Register the render startup hook
  let mut js_hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  js_hooks.render_startup.tap(render_startup::new(self));

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
  let entry_module_count = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey);

  if entry_module_count == 0 {
    return Ok(());
  }

  let tree_runtime_requirements =
    ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey);

  if tree_runtime_requirements.contains(RuntimeGlobals::STARTUP)
    || tree_runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT)
  {
    return Ok(());
  }

  let mut startup_with_call = ConcatSource::default();

  startup_with_call.add(RawStringSource::from(
    "\n// Entrypoint: appended startup call because none was added automatically\n",
  ));
  startup_with_call.add(RawStringSource::from(format!(
    "{}();\n",
    RuntimeGlobals::STARTUP.name()
  )));
  startup_with_call.add(render_source.source.clone());
  render_source.source = startup_with_call.boxed();
  Ok(())
}

impl Plugin for EmbedFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "EmbedFederationRuntimePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements_tree::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirement_in_tree::new(self));
    Ok(())
  }
}

impl Default for EmbedFederationRuntimePlugin {
  fn default() -> Self {
    Self::new()
  }
}
