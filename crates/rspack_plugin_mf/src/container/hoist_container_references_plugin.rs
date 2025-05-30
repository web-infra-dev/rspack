//! # HoistContainerReferencesPlugin
//!
//! Optimizes Module Federation chunk placement by hoisting container references and
//! their dependencies to runtime chunks. Recursively collects referenced modules
//! and moves them to appropriate runtime chunks for proper federation execution.

use std::{
  collections::{HashSet, VecDeque},
  sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, Compilation, CompilationOptimizeChunks, CompilerCompilation, CompilerOptions,
  Dependency, DependencyId, Module, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::{
  container_entry_dependency::ContainerEntryDependency, fallback_dependency::FallbackDependency,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  remote_to_external_dependency::RemoteToExternalDependency,
};

#[plugin]
#[derive(Debug, Default)]
pub struct HoistContainerReferencesPlugin {
  container_entry_deps: Arc<Mutex<HashSet<DependencyId>>>,
  federation_runtime_deps: Arc<Mutex<HashSet<DependencyId>>>,
  remote_deps: Arc<Mutex<HashSet<DependencyId>>>,
}

// Structs for hook handlers
struct ContainerEntryDepCollector {
  set: Arc<Mutex<HashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddContainerEntryDependencyHook
  for ContainerEntryDepCollector
{
  async fn run(&self, dep: &ContainerEntryDependency) -> Result<()> {
    self
      .set
      .lock()
      .expect("Failed to lock container entry deps")
      .insert(*dep.id());
    Ok(())
  }
}

struct FederationRuntimeDepCollector {
  set: Arc<Mutex<HashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddFederationRuntimeDependencyHook
  for FederationRuntimeDepCollector
{
  async fn run(&self, dep: &FederationRuntimeDependency) -> Result<()> {
    self
      .set
      .lock()
      .expect("Failed to lock federation runtime deps")
      .insert(*dep.id());
    Ok(())
  }
}

struct RemoteDepCollector {
  set: Arc<Mutex<HashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddRemoteDependencyHook for RemoteDepCollector {
  async fn run(&self, dep: &dyn Dependency) -> Result<()> {
    if let Some(dep) = dep.downcast_ref::<RemoteToExternalDependency>() {
      self
        .set
        .lock()
        .expect("Failed to lock remote deps")
        .insert(*dep.id());
    }
    if let Some(dep) = dep.downcast_ref::<FallbackDependency>() {
      self
        .set
        .lock()
        .expect("Failed to lock remote deps")
        .insert(*dep.id());
    }
    Ok(())
  }
}

#[plugin_hook(CompilerCompilation for HoistContainerReferencesPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut rspack_core::CompilationParams,
) -> Result<()> {
  let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

  hooks
    .add_container_entry_dependency
    .lock()
    .await
    .tap(ContainerEntryDepCollector {
      set: Arc::clone(&self.container_entry_deps),
    });

  hooks
    .add_federation_runtime_dependency
    .lock()
    .await
    .tap(FederationRuntimeDepCollector {
      set: Arc::clone(&self.federation_runtime_deps),
    });

  hooks
    .add_remote_dependency
    .lock()
    .await
    .tap(RemoteDepCollector {
      set: Arc::clone(&self.remote_deps),
    });

  Ok(())
}

#[plugin_hook(CompilationOptimizeChunks for HoistContainerReferencesPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED + 1)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  // Helper: recursively collect all referenced modules (matching TypeScript implementation)
  fn get_all_referenced_modules(
    compilation: &Compilation,
    module: &dyn Module,
    ty: &str,
  ) -> HashSet<ModuleIdentifier> {
    let mut collected = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = VecDeque::new();

    let module_id = module.identifier();
    collected.insert(module_id);
    visited.insert(module_id);
    stack.push_back(module_id);

    while let Some(current_module_id) = stack.pop_front() {
      let module_graph = compilation.get_module_graph();

      for conn in module_graph.get_outgoing_connections(&current_module_id) {
        let connected_id = *conn.module_identifier();

        // Skip if module has already been visited
        if visited.contains(&connected_id) {
          continue;
        }

        // Handle 'initial' type (skipping async blocks)
        if ty == "initial" {
          let parent_block = module_graph.get_parent_block(&conn.dependency_id);
          if parent_block.is_some() {
            // Skip async blocks for "initial"
            continue;
          }
        }

        // Add connected module to collection and stack
        collected.insert(connected_id);
        visited.insert(connected_id);
        stack.push_back(connected_id);
      }
    }

    collected
  }

  // Helper: get runtime chunks from entrypoints (matching TypeScript implementation)
  fn get_runtime_chunks(compilation: &Compilation) -> HashSet<rspack_core::ChunkUkey> {
    let mut runtime_chunks = HashSet::new();
    for entrypoint_ukey in compilation.entrypoints.values() {
      if let Some(entrypoint) = compilation.chunk_group_by_ukey.get(entrypoint_ukey) {
        let runtime_chunk = entrypoint.get_runtime_chunk(&compilation.chunk_group_by_ukey);
        runtime_chunks.insert(runtime_chunk);
      }
    }
    runtime_chunks
  }

  // Helper: clean up chunks by disconnecting unused modules (matching TypeScript implementation)
  fn clean_up_chunks(compilation: &mut Compilation, modules: &mut HashSet<ModuleIdentifier>) {
    for module_id in modules.iter() {
      let chunks_vec: Vec<_> = compilation
        .chunk_graph
        .get_module_chunks(*module_id)
        .iter()
        .copied()
        .collect();

      for chunk_ukey in chunks_vec {
        let chunk = compilation.chunk_by_ukey.get(&chunk_ukey);
        let has_runtime = chunk.is_some_and(|c| c.has_runtime(&compilation.chunk_group_by_ukey));

        if !has_runtime {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&chunk_ukey, *module_id);

          if compilation
            .chunk_graph
            .get_number_of_chunk_modules(&chunk_ukey)
            == 0
            && compilation
              .chunk_graph
              .get_number_of_entry_modules(&chunk_ukey)
              == 0
          {
            compilation.chunk_graph.remove_chunk(&chunk_ukey);
            let removed_chunk = compilation.chunk_by_ukey.remove(&chunk_ukey);

            // Remove from named chunks if it has a name
            if let Some(chunk) = removed_chunk {
              if let Some(name) = chunk.name() {
                compilation.named_chunks.remove(name);
              }
            }
          }
        }
      }
    }
    modules.clear();
  }

  let _runtime_chunks = get_runtime_chunks(compilation);
  let mut all_modules_to_hoist = HashSet::new();

  // Process container entry dependencies
  for dep_id in self
    .container_entry_deps
    .lock()
    .expect("Failed to lock container entry deps")
    .iter()
  {
    let module_graph = compilation.get_module_graph();
    if let Some(module_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        let referenced_modules =
          get_all_referenced_modules(compilation, module.as_ref(), "initial");
        all_modules_to_hoist.extend(&referenced_modules);

        // Get module runtimes and hoist to corresponding runtime chunks
        let runtime_specs: Vec<_> = compilation
          .chunk_graph
          .get_module_runtimes_iter(*module_id, &compilation.chunk_by_ukey)
          .cloned()
          .collect();

        for runtime_spec in runtime_specs {
          // Find runtime chunks by name - iterate over each runtime in the spec
          for runtime_name in runtime_spec.iter() {
            if let Some(runtime_chunk) = compilation.named_chunks.get(runtime_name.as_ref()) {
              for &ref_module_id in &referenced_modules {
                if !compilation
                  .chunk_graph
                  .is_module_in_chunk(&ref_module_id, *runtime_chunk)
                {
                  compilation
                    .chunk_graph
                    .connect_chunk_and_module(*runtime_chunk, ref_module_id);
                }
              }
            }
          }
        }
      }
    }
  }

  // Process federation runtime dependencies
  for dep_id in self
    .federation_runtime_deps
    .lock()
    .expect("Failed to lock federation runtime deps")
    .iter()
  {
    let module_graph = compilation.get_module_graph();
    if let Some(module_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        let referenced_modules =
          get_all_referenced_modules(compilation, module.as_ref(), "initial");
        all_modules_to_hoist.extend(&referenced_modules);

        // Get module runtimes and hoist to corresponding runtime chunks
        let runtime_specs: Vec<_> = compilation
          .chunk_graph
          .get_module_runtimes_iter(*module_id, &compilation.chunk_by_ukey)
          .cloned()
          .collect();

        for runtime_spec in runtime_specs {
          // Find runtime chunks by name - iterate over each runtime in the spec
          for runtime_name in runtime_spec.iter() {
            if let Some(runtime_chunk) = compilation.named_chunks.get(runtime_name.as_ref()) {
              for &ref_module_id in &referenced_modules {
                if !compilation
                  .chunk_graph
                  .is_module_in_chunk(&ref_module_id, *runtime_chunk)
                {
                  compilation
                    .chunk_graph
                    .connect_chunk_and_module(*runtime_chunk, ref_module_id);
                }
              }
            }
          }
        }
      }
    }
  }

  // Process remote dependencies
  for dep_id in self
    .remote_deps
    .lock()
    .expect("Failed to lock remote deps")
    .iter()
  {
    let module_graph = compilation.get_module_graph();
    if let Some(module_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        let referenced_modules =
          get_all_referenced_modules(compilation, module.as_ref(), "initial");
        all_modules_to_hoist.extend(&referenced_modules);

        // Get module runtimes and hoist to corresponding runtime chunks
        let runtime_specs: Vec<_> = compilation
          .chunk_graph
          .get_module_runtimes_iter(*module_id, &compilation.chunk_by_ukey)
          .cloned()
          .collect();

        for runtime_spec in runtime_specs {
          // Find runtime chunks by name - iterate over each runtime in the spec
          for runtime_name in runtime_spec.iter() {
            if let Some(runtime_chunk) = compilation.named_chunks.get(runtime_name.as_ref()) {
              for &ref_module_id in &referenced_modules {
                if !compilation
                  .chunk_graph
                  .is_module_in_chunk(&ref_module_id, *runtime_chunk)
                {
                  compilation
                    .chunk_graph
                    .connect_chunk_and_module(*runtime_chunk, ref_module_id);
                }
              }
            }
          }
        }
      }
    }
  }

  // Cleanup: disconnect hoisted modules from non-runtime chunks (matching TypeScript implementation)
  clean_up_chunks(compilation, &mut all_modules_to_hoist);

  Ok(None)
}

impl Plugin for HoistContainerReferencesPlugin {
  fn name(&self) -> &'static str {
    "HoistContainerReferencesPlugin"
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
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
