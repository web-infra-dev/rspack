//! # ModuleFederationRuntimePlugin
//!
//! Main orchestration plugin for Module Federation runtime functionality.
//! Coordinates federation plugins, manages runtime dependencies, and adds the base FederationRuntimeModule.

use rspack_cacheable::cacheable;
use rspack_core::{
  BoxDependency, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationOptimizeChunkModules, CompilationParams, CompilerCompilation, CompilerFinishMake,
  DependencyType, EntryOptions, Plugin, RuntimeGlobals, RuntimeModule, SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Deserialize;

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_plugin::EmbedFederationRuntimePlugin,
  federation_data_runtime_module::FederationDataRuntimeModule,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  hoist_container_references_plugin::HoistContainerReferencesPlugin,
};
use crate::{ShareScope, sharing::consume_shared_module::ConsumeSharedModule};

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModuleFederationRuntimePluginOptions {
  pub entry_runtime: Option<String>,
  #[serde(default)]
  pub experiments: ModuleFederationRuntimeExperimentsOptions,
}

#[cacheable]
#[derive(Debug, Default, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct ModuleFederationRuntimeExperimentsOptions {
  #[serde(default)]
  pub async_startup: bool,
}

impl ModuleFederationRuntimeExperimentsOptions {
  /// Whether the startup of `chunk_ukey` must be asynchronous.
  ///
  /// Ordered (array) share scopes are initialized before their consumes are
  /// installed, so a runtime whose initial chunks contain such a consume must
  /// await that initialization. This is derived from the actual initial
  /// `ConsumeSharedModule`s rather than only from the high-level `shared`
  /// option, so consumes contributed by a separately installed enhanced
  /// `ConsumeSharedPlugin` are covered too. Scalar scopes keep synchronous
  /// startup.
  ///
  /// The decision is owned by the runtime chunk: the runtime installs the
  /// initial consumes of every entry it serves in one call, so an entry chunk
  /// that delegates to a shared runtime (`runtimeChunk: 'single'`) must take
  /// the same mode as that runtime even when its own initial chunks contain
  /// no ordered consume.
  pub fn needs_async_startup(&self, compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
    if self.async_startup {
      return true;
    }
    let artifact = &compilation.build_chunk_graph_artifact;
    let chunk = artifact.chunk_by_ukey.expect_get(chunk_ukey);
    let mut runtime_chunks: Vec<ChunkUkey> = chunk
      .groups()
      .iter()
      .filter_map(|group_ukey| artifact.chunk_group_by_ukey.get(group_ukey))
      .filter(|group| group.kind.is_entrypoint())
      .map(|group| group.get_runtime_chunk(&artifact.chunk_group_by_ukey))
      .collect();
    if runtime_chunks.is_empty() {
      runtime_chunks.push(*chunk_ukey);
    }
    runtime_chunks.sort_unstable();
    runtime_chunks.dedup();

    let module_graph = compilation.get_module_graph();
    runtime_chunks
      .iter()
      .flat_map(|runtime_chunk| {
        artifact
          .chunk_by_ukey
          .expect_get(runtime_chunk)
          .get_all_initial_chunks(&artifact.chunk_group_by_ukey)
      })
      .flat_map(|chunk| {
        artifact
          .chunk_graph
          .get_chunk_modules_identifier_by_source_type(
            &chunk,
            SourceType::ConsumeShared,
            module_graph,
          )
      })
      .any(|module_identifier| {
        module_graph
          .module_by_identifier(&module_identifier)
          .and_then(|module| module.as_any().downcast_ref::<ConsumeSharedModule>())
          .is_some_and(|module| matches!(module.share_scope(), ShareScope::Multiple(_)))
      })
  }
}

#[plugin]
#[derive(Debug)]
pub struct ModuleFederationRuntimePlugin {
  options: ModuleFederationRuntimePluginOptions,
}

impl ModuleFederationRuntimePlugin {
  pub fn new(options: ModuleFederationRuntimePluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilerCompilation for ModuleFederationRuntimePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::FederationRuntime,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  // Add base FederationRuntimeModule which is responsible for providing bundler data to the runtime.
  runtime_modules.push(Box::new(FederationDataRuntimeModule::new(
    &compilation.runtime_template,
  )));

  Ok(())
}

#[plugin_hook(CompilerFinishMake for ModuleFederationRuntimePlugin, stage = 1000)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  if let Some(entry_request) = self.options.entry_runtime.clone() {
    let federation_runtime_dep = FederationRuntimeDependency::new(entry_request.clone());

    let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

    hooks
      .add_federation_runtime_dependency
      .lock()
      .await
      .call(&federation_runtime_dep)
      .await?;

    let boxed_dep = BoxDependency::new(federation_runtime_dep);
    let entry_options = EntryOptions::default();
    let args = vec![(boxed_dep, entry_options)];

    compilation.add_include(args).await?;
  }

  Ok(())
}

// Container startup can return a promise independently of module-level await.
// Wait for the chunk graph so async-only consumes and separate runtimes do not
// make synchronous containers acquire top-level await in module-library output.
#[plugin_hook(CompilationOptimizeChunkModules for ModuleFederationRuntimePlugin, stage = -1)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let async_containers: Vec<_> = module_graph
    .modules()
    .filter(|(_, module)| {
      module
        .as_ref()
        .as_any()
        .downcast_ref::<ContainerEntryModule>()
        .is_some()
    })
    .filter(|(module_identifier, _)| {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(**module_identifier)
        .iter()
        .any(|chunk| self.options.experiments.needs_async_startup(compilation, chunk))
    })
    .map(|(module_identifier, _)| *module_identifier)
    .collect();
  compilation.async_modules_artifact.extend(async_containers);
  Ok(None)
}

impl Plugin for ModuleFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.container.ModuleFederationRuntimePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));

    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    // Apply supporting plugins
    EmbedFederationRuntimePlugin::new(self.options.experiments.clone()).apply(ctx)?;
    HoistContainerReferencesPlugin::default().apply(ctx)?;

    Ok(())
  }
}
