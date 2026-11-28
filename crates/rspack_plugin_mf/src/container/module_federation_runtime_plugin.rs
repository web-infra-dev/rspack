//! # ModuleFederationRuntimePlugin
//!
//! Main orchestration plugin for Module Federation runtime functionality.
//! Coordinates federation plugins, manages runtime dependencies, and adds the base FederationRuntimeModule.

use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
};

use rspack_cacheable::cacheable;
use rspack_collections::IdentifierSet;
use rspack_core::{
  AsyncModulesArtifact, BoxDependency, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationFinishModules, CompilationId,
  CompilationOptimizeChunks, CompilationParams, CompilerCompilation, CompilerFinishMake,
  Dependency, DependencyId, DependencyType, EntryOptions, ExportsInfoArtifact, ModuleIdentifier,
  Plugin, RuntimeGlobals, RuntimeModule, RuntimeSpec, SideEffectsStateArtifact, SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;
use rustc_hash::FxHashSet;
use serde::Deserialize;

use super::{
  container_entry_module::ContainerEntryModule,
  embed_federation_runtime_plugin::EmbedFederationRuntimePlugin,
  federation_data_runtime_module::FederationDataRuntimeModule,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  hoist_container_references_plugin::HoistContainerReferencesPlugin,
};

/// Scope in which an unused bundler runtime include may be omitted.
#[derive(Debug, Default)]
pub enum BundlerRuntimeExclusion {
  #[default]
  Disabled,
  All,
  EntryName(RspackRegex),
}

impl BundlerRuntimeExclusion {
  pub fn is_enabled(&self) -> bool {
    !matches!(self, Self::Disabled)
  }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModuleFederationRuntimePluginOptions {
  pub entry_runtime: Option<String>,
  #[serde(skip)]
  pub remove_unused_runtime: Arc<BundlerRuntimeExclusion>,
  #[serde(default)]
  pub experiments: ModuleFederationRuntimeExperimentsOptions,
}

#[cacheable]
#[derive(Debug, Default, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct ModuleFederationRuntimeExperimentsOptions {
  #[serde(default)]
  pub async_startup: bool,
}

#[plugin]
#[derive(Debug)]
pub struct ModuleFederationRuntimePlugin {
  options: ModuleFederationRuntimePluginOptions,
  latest_runtime_dep: Mutex<Option<(CompilationId, DependencyId)>>,
}

impl ModuleFederationRuntimePlugin {
  pub fn new(options: ModuleFederationRuntimePluginOptions) -> Self {
    Self::new_inner(options, Mutex::new(None))
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
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  if !crate::utils::has_federation_runtime(compilation, chunk_ukey) {
    return Ok(());
  }
  // Add base FederationRuntimeModule which is responsible for providing bundler data to the runtime.
  runtime_modules.push(Box::new(FederationDataRuntimeModule::new(
    &compilation.runtime_template,
  )));

  Ok(())
}

#[plugin_hook(CompilerFinishMake for ModuleFederationRuntimePlugin, stage = 1000)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  if let Some(entry_request) = self.options.entry_runtime.clone() {
    let federation_runtime_dep = FederationRuntimeDependency::new(entry_request);
    let dep_id = *federation_runtime_dep.id();

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

    *self
      .latest_runtime_dep
      .lock()
      .expect("Failed to lock latest_runtime_dep") = Some((compilation.id(), dep_id));
    compilation.add_include(args).await?;
  }

  Ok(())
}

/// Remove the federation bundler runtime and provider-only modules from entrypoints
/// and Workers that have no `Expose` or `Remote` modules (within the opt-in scope).
#[plugin_hook(CompilationOptimizeChunks for ModuleFederationRuntimePlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let Some(runtime_dep_id) = self
    .latest_runtime_dep
    .lock()
    .expect("Failed to lock latest_runtime_dep")
    .and_then(|(comp_id, dep_id)| (comp_id == compilation.id()).then_some(dep_id))
  else {
    return Ok(None);
  };

  let mg = compilation.get_module_graph();
  let Some(&runtime_module) = mg.module_identifier_by_dependency_id(&runtime_dep_id) else {
    return Ok(None);
  };

  let mut other_runtime_modules = Vec::new();
  let mut provider_modules = Vec::new();
  for dep in &compilation.global_entry.include_dependencies {
    if *dep == runtime_dep_id {
      continue;
    }
    match mg.dependency_by_id(dep).dependency_type() {
      DependencyType::FederationRuntime => {
        if let Some(module) = mg.module_identifier_by_dependency_id(dep) {
          other_runtime_modules.push(*module);
        }
      }
      DependencyType::ProvideSharedModule => {
        if let Some(module) = mg.module_identifier_by_dependency_id(dep) {
          provider_modules.push(*module);
        }
      }
      _ => {}
    }
  }
  provider_modules.sort_unstable();
  provider_modules.dedup();

  let artifact = &compilation.build_chunk_graph_artifact;
  let mut required_runtimes = RuntimeSpec::default();

  match self.options.remove_unused_runtime.as_ref() {
    BundlerRuntimeExclusion::Disabled => return Ok(None),
    BundlerRuntimeExclusion::All => {
      // Automatic mode: retain federation runtime for any runtime that reaches
      // an Expose (container) or Remote module.
      let mut seen_chunks = FxHashSet::default();
      for (mid, module) in mg.modules() {
        if module.as_normal_module().is_some() {
          continue;
        }
        if !module
          .source_types(mg)
          .iter()
          .any(|st| matches!(st, SourceType::Expose | SourceType::Remote))
        {
          continue;
        }
        for chunk in artifact.chunk_graph.get_module_chunks(*mid) {
          if seen_chunks.insert(*chunk) {
            required_runtimes.extend(artifact.chunk_by_ukey.expect_get(chunk).runtime());
          }
        }
      }
    }
    BundlerRuntimeExclusion::EntryName(regex) => {
      // Explicit RegExp mode: honor the user's pattern directly without scanning
      // Expose/Remote modules. Entries and named Workers that do not match
      // (as well as unnamed Workers) retain their runtime.
      for (name, group_ukey) in artifact
        .entrypoints
        .iter()
        .map(|(name, group)| (Some(name.as_str()), group))
        .chain(artifact.async_entrypoints.iter().map(|group| {
          (
            artifact
              .chunk_group_by_ukey
              .expect_get(group)
              .kind
              .get_entry_options()
              .and_then(|options| options.name.as_deref()),
            group,
          )
        }))
      {
        if !name.is_some_and(|name| regex.test(name)) {
          let entry_chunk = artifact
            .chunk_group_by_ukey
            .expect_get(group_ukey)
            .get_entrypoint_chunk();
          required_runtimes.extend(artifact.chunk_by_ukey.expect_get(&entry_chunk).runtime());
        }
      }
    }
  }

  // Find chunks that contain our runtime or provider modules and do not require federation.
  let mut target_chunks = FxHashSet::default();
  for root in std::iter::once(&runtime_module).chain(&provider_modules) {
    for chunk_ukey in artifact.chunk_graph.get_module_chunks(*root) {
      if target_chunks.contains(chunk_ukey) {
        continue;
      }
      let chunk_runtime = artifact.chunk_by_ukey.expect_get(chunk_ukey).runtime();
      if !chunk_runtime
        .iter()
        .any(|runtime| required_runtimes.contains(runtime))
      {
        target_chunks.insert(*chunk_ukey);
      }
    }
  }

  if target_chunks.is_empty() {
    return Ok(None);
  }

  let mut disconnects: Vec<(ChunkUkey, ModuleIdentifier)> = Vec::new();
  let mut queue = VecDeque::new();
  let mut mf_candidates = IdentifierSet::default();
  let mut protected = IdentifierSet::default();

  for chunk_ukey in target_chunks {
    let chunk_runtime = artifact.chunk_by_ukey.expect_get(&chunk_ukey).runtime();
    let has_other_runtime = other_runtime_modules
      .iter()
      .any(|m| artifact.chunk_graph.is_module_in_chunk(m, chunk_ukey));

    mf_candidates.clear();
    protected.clear();
    queue.clear();

    if artifact
      .chunk_graph
      .is_module_in_chunk(&runtime_module, chunk_ukey)
    {
      mf_candidates.insert(runtime_module);
      queue.push_back(runtime_module);
    }
    if !has_other_runtime {
      for provider in &provider_modules {
        if artifact
          .chunk_graph
          .is_module_in_chunk(provider, chunk_ukey)
          && mf_candidates.insert(*provider)
        {
          queue.push_back(*provider);
        }
      }
    }

    if mf_candidates.is_empty() {
      continue;
    }

    // Collect the federation runtime / provider subgraph within this chunk.
    while let Some(curr) = queue.pop_front() {
      for conn in mg.get_outgoing_connections(&curr) {
        let target = *conn.module_identifier();
        if artifact.chunk_graph.is_module_in_chunk(&target, chunk_ukey)
          && mf_candidates.insert(target)
        {
          queue.push_back(target);
        }
      }
    }

    // Protect any candidate that is an entry module or referenced by non-candidate modules
    // in this chunk or in another chunk sharing this runtime (e.g. async child chunks).
    for (entry_module, _) in artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(&chunk_ukey)
    {
      if mf_candidates.contains(entry_module) && protected.insert(*entry_module) {
        queue.push_back(*entry_module);
      }
    }

    for &candidate in &mf_candidates {
      if protected.contains(&candidate) {
        continue;
      }
      let referenced_externally = mg.get_incoming_connections(&candidate).any(|conn| {
        if let Some(orig) = conn.original_module_identifier {
          if mf_candidates.contains(&orig) {
            return false;
          }
          artifact
            .chunk_graph
            .get_module_chunks(orig)
            .iter()
            .any(|orig_chunk| {
              *orig_chunk == chunk_ukey
                || artifact
                  .chunk_by_ukey
                  .expect_get(orig_chunk)
                  .runtime()
                  .iter()
                  .any(|r| chunk_runtime.contains(r))
            })
        } else {
          if conn.dependency_id == runtime_dep_id {
            return false;
          }
          if !has_other_runtime
            && mg.dependency_by_id(&conn.dependency_id).dependency_type()
              == &DependencyType::ProvideSharedModule
          {
            return false;
          }
          true
        }
      });
      if referenced_externally && protected.insert(candidate) {
        queue.push_back(candidate);
      }
    }

    // Propagate protection through outgoing edges inside mf_candidates.
    while let Some(curr) = queue.pop_front() {
      for conn in mg.get_outgoing_connections(&curr) {
        let target = *conn.module_identifier();
        if mf_candidates.contains(&target) && protected.insert(target) {
          queue.push_back(target);
        }
      }
    }

    for &candidate in &mf_candidates {
      if !protected.contains(&candidate) {
        disconnects.push((chunk_ukey, candidate));
      }
    }
  }

  for (chunk_ukey, module) in disconnects {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .disconnect_chunk_and_module(&chunk_ukey, module);
  }

  Ok(None)
}

// When MF async startup is enabled, STARTUP may resolve asynchronously even if the entry module
// itself doesn't have top-level await. Mark MF container entry modules as async so downstream
// renderers (e.g. library wrappers) can safely `await` exports without sprinkling MF-specific
// RuntimeGlobals checks across generic plugins.
#[plugin_hook(CompilationFinishModules for ModuleFederationRuntimePlugin, stage = 1000)]
async fn finish_modules(
  &self,
  compilation: &Compilation,
  async_modules_artifact: &mut AsyncModulesArtifact,
  _exports_info_artifact: &mut ExportsInfoArtifact,
  _side_effects_state_artifact: &mut SideEffectsStateArtifact,
) -> Result<()> {
  if !self.options.experiments.async_startup {
    return Ok(());
  }

  let module_graph = compilation.get_module_graph();
  for (module_identifier, module) in module_graph.modules() {
    if module
      .as_ref()
      .as_any()
      .downcast_ref::<ContainerEntryModule>()
      .is_some()
    {
      async_modules_artifact.insert(*module_identifier);
    }
  }

  Ok(())
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
      .finish_modules
      .tap(finish_modules::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    if self.options.remove_unused_runtime.is_enabled() {
      ctx
        .compilation_hooks
        .optimize_chunks
        .tap(optimize_chunks::new(self));
    }

    // Apply supporting plugins
    EmbedFederationRuntimePlugin::new(self.options.experiments.clone()).apply(ctx)?;
    HoistContainerReferencesPlugin::default().apply(ctx)?;

    Ok(())
  }
}
