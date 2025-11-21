//! # HoistContainerReferencesPlugin
//!
//! Optimizes Module Federation chunk placement by hoisting container references and
//! their dependencies to runtime chunks. This plugin enhances module hoisting and
//! runtime chunk handling for Module Federation by:
//!
//! - Separating dependency handling for container, federation runtime, and remote modules
//! - Enhanced runtime chunk detection supporting `runtimeChunk: 'single'` configurations
//! - Recursive collection of referenced modules with proper async dependency exclusion
//! - Efficient cleanup of empty non-runtime chunks after hoisting
//!
//! The plugin coordinates with FederationModulesPlugin through a hook-based system
//! to collect and manage federation-specific dependencies across the compilation.

use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rspack_core::{
  Compilation, CompilationOptimizeChunks, CompilerCompilation, Dependency, DependencyId,
  ModuleIdentifier, Plugin, RuntimeSpec, incremental::Mutation,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  container_entry_dependency::ContainerEntryDependency, fallback_dependency::FallbackDependency,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  remote_to_external_dependency::RemoteToExternalDependency,
};

#[plugin]
#[derive(Debug, Default)]
pub struct HoistContainerReferencesPlugin {
  federation_deps: Arc<Mutex<FxHashSet<DependencyId>>>,
}

struct ContainerEntryDepCollector {
  set: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddContainerEntryDependencyHook
  for ContainerEntryDepCollector
{
  async fn run(&self, dep: &ContainerEntryDependency) -> Result<()> {
    self
      .set
      .lock()
      .expect("Failed to lock federation deps")
      .insert(*dep.id());
    Ok(())
  }
}

struct FederationRuntimeDepCollector {
  set: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddFederationRuntimeDependencyHook
  for FederationRuntimeDepCollector
{
  async fn run(&self, dep: &FederationRuntimeDependency) -> Result<()> {
    self
      .set
      .lock()
      .expect("Failed to lock federation deps")
      .insert(*dep.id());
    Ok(())
  }
}

struct RemoteDepCollector {
  set: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddRemoteDependencyHook for RemoteDepCollector {
  async fn run(&self, dep: &dyn Dependency) -> Result<()> {
    if let Some(dep) = dep.downcast_ref::<RemoteToExternalDependency>() {
      self
        .set
        .lock()
        .expect("Failed to lock federation deps")
        .insert(*dep.id());
    }
    if let Some(dep) = dep.downcast_ref::<FallbackDependency>() {
      self
        .set
        .lock()
        .expect("Failed to lock federation deps")
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
      set: Arc::clone(&self.federation_deps),
    });

  hooks
    .add_federation_runtime_dependency
    .lock()
    .await
    .tap(FederationRuntimeDepCollector {
      set: Arc::clone(&self.federation_deps),
    });

  hooks
    .add_remote_dependency
    .lock()
    .await
    .tap(RemoteDepCollector {
      set: Arc::clone(&self.federation_deps),
    });

  Ok(())
}

#[plugin_hook(CompilationOptimizeChunks for HoistContainerReferencesPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED + 1)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  // Helper: recursively collect all referenced modules
  fn get_all_referenced_modules(
    compilation: &Compilation,
    module_id: ModuleIdentifier,
    ty: &str,
  ) -> FxHashSet<ModuleIdentifier> {
    let mut collected = FxHashSet::default();
    let mut visited = FxHashSet::default();
    let mut stack = VecDeque::new();

    collected.insert(module_id);
    visited.insert(module_id);
    stack.push_back(module_id);

    while let Some(current_module_id) = stack.pop_front() {
      let module_graph = compilation.get_module_graph();

      for conn in module_graph.get_outgoing_connections(&current_module_id) {
        let connected_id = *conn.module_identifier();

        // Skip if already visited
        if visited.contains(&connected_id) {
          continue;
        }

        // Handle 'initial' type - skip async blocks
        if ty == "initial" {
          let parent_block = module_graph.get_parent_block(&conn.dependency_id);
          if parent_block.is_some() {
            continue;
          }
        }

        // Add to collection and stack
        collected.insert(connected_id);
        visited.insert(connected_id);
        stack.push_back(connected_id);
      }
    }

    collected
  }

  let mg = compilation.get_module_graph();

  // Collect all federation (container, runtime, and remote) referenced modules
  let all_modules_to_hoist = self
    .federation_deps
    .lock()
    .expect("Failed to lock federation deps")
    .iter()
    .filter_map(|dep| mg.module_identifier_by_dependency_id(dep))
    .flat_map(|module| get_all_referenced_modules(compilation, *module, "initial"))
    .collect::<FxHashSet<_>>();

  // Hoist referenced modules to their runtime chunk
  let entries = compilation
    .get_chunk_graph_entries()
    .filter_map(|entry| {
      compilation
        .chunk_by_ukey
        .get(&entry)
        .map(|chunk| (chunk.runtime(), entry))
    })
    .collect::<FxHashMap<_, _>>();
  for module in &all_modules_to_hoist {
    let runtime_chunks = compilation
      .chunk_graph
      .get_module_runtimes_iter(*module, &compilation.chunk_by_ukey)
      .flat_map(|runtime| runtime.iter())
      .filter_map(|runtime| entries.get(&RuntimeSpec::from_iter([*runtime])).copied())
      .collect::<Vec<_>>();
    for runtime_chunk in runtime_chunks {
      if !compilation
        .chunk_graph
        .is_module_in_chunk(module, runtime_chunk)
      {
        compilation
          .chunk_graph
          .connect_chunk_and_module(runtime_chunk, *module);
      }
    }
  }

  // Disconnect hoisted modules from non-runtime chunks, this is safe since we already hoist them to runtime chunk
  let runtime_chunks = entries.values().copied().collect::<FxHashSet<_>>();
  for module in all_modules_to_hoist {
    let non_runtime_chunks = compilation
      .chunk_graph
      .get_module_chunks(module)
      .iter()
      .filter(|chunk| !runtime_chunks.contains(chunk))
      .copied()
      .collect::<Vec<_>>();
    for chunk in non_runtime_chunks {
      compilation
        .chunk_graph
        .disconnect_chunk_and_module(&chunk, module);

      if compilation.chunk_graph.get_number_of_chunk_modules(&chunk) == 0
        && compilation.chunk_graph.get_number_of_entry_modules(&chunk) == 0
        && let Some(mut removed_chunk) = compilation.chunk_by_ukey.remove(&chunk)
      {
        compilation
          .chunk_graph
          .disconnect_chunk(&mut removed_chunk, &mut compilation.chunk_group_by_ukey);
        compilation.chunk_graph.remove_chunk(&chunk);

        // Remove from named chunks if it has a name
        if let Some(name) = removed_chunk.name() {
          compilation.named_chunks.remove(name);
        }
        // Record mutation
        if let Some(mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkRemove { chunk });
        }
      }
    }
  }

  Ok(None)
}

impl Plugin for HoistContainerReferencesPlugin {
  fn name(&self) -> &'static str {
    "HoistContainerReferencesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
