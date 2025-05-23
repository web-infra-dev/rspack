use std::{
  collections::{HashSet, VecDeque},
  sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationOptimizeChunks, CompilerCompilation,
  CompilerOptions, Dependency, DependencyId, Module, ModuleIdentifier, Plugin, PluginContext,
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
    self.set.lock().unwrap().insert(dep.id().clone());
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
    self.set.lock().unwrap().insert(dep.id().clone());
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
      self.set.lock().unwrap().insert(dep.id().clone());
    }
    if let Some(dep) = dep.downcast_ref::<FallbackDependency>() {
      self.set.lock().unwrap().insert(dep.id().clone());
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

#[plugin_hook(CompilationOptimizeChunks for HoistContainerReferencesPlugin)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  // Helper: recursively collect all referenced modules (skipping async blocks for "initial" type)
  fn get_all_referenced_modules(
    compilation: &Compilation,
    module: &dyn Module,
    ty: &str,
    visited: &mut HashSet<ModuleIdentifier>,
  ) -> HashSet<ModuleIdentifier> {
    let mut collected = HashSet::new();
    let mut stack = VecDeque::new();
    stack.push_back(module.identifier());

    while let Some(module_id) = stack.pop_front() {
      if !visited.insert(module_id) {
        continue;
      }
      collected.insert(module_id);
      let module_graph = compilation.get_module_graph();
      if let Some(module) = module_graph.module_by_identifier(&module_id) {
        for conn in module_graph.get_outgoing_connections(&module_id) {
          let connected_id = conn.module_identifier();
          if ty == "initial" {
            let parent_block = module_graph.get_parent_block(&conn.dependency_id);
            if parent_block.is_some() {
              // skip async blocks for "initial"
              continue;
            }
          }
          stack.push_back(*connected_id);
        }
      }
    }
    collected
  }

  // Get all runtime chunks (entrypoint runtime chunks)
  let runtime_chunks: HashSet<ChunkUkey> = compilation
    .entrypoints
    .values()
    .filter_map(|entrypoint_ukey| {
      compilation
        .chunk_group_by_ukey
        .get(entrypoint_ukey)
        .map(|group| group.get_runtime_chunk(&compilation.chunk_group_by_ukey))
    })
    .collect();

  // Hoist referenced modules to runtime chunks
  let mut all_modules_to_hoist = HashSet::new();
  for dep_id in self
    .container_entry_deps
    .lock()
    .unwrap()
    .iter()
    .chain(self.federation_runtime_deps.lock().unwrap().iter())
    .chain(self.remote_deps.lock().unwrap().iter())
  {
    let module_graph = compilation.get_module_graph();
    if let Some(module_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
      if let Some(module) = module_graph.module_by_identifier(&module_id) {
        let referenced =
          get_all_referenced_modules(compilation, module.as_ref(), "initial", &mut HashSet::new());
        all_modules_to_hoist.extend(&referenced);
        for module_id in &referenced {
          for &runtime_chunk in &runtime_chunks {
            if !compilation
              .chunk_graph
              .is_module_in_chunk(module_id, runtime_chunk)
            {
              compilation
                .chunk_graph
                .connect_chunk_and_module(runtime_chunk, *module_id);
            }
          }
        }
      }
    }
  }

  // Cleanup: disconnect hoisted modules from non-runtime chunks and remove empty chunks
  for module_id in &all_modules_to_hoist {
    let chunks_vec: Vec<_> = compilation
      .chunk_graph
      .get_module_chunks(*module_id)
      .iter()
      .copied()
      .collect();
    for chunk_ukey in chunks_vec {
      if !runtime_chunks.contains(&chunk_ukey) {
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
          compilation.chunk_by_ukey.remove(&chunk_ukey);
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

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, options: &CompilerOptions) -> Result<()> {
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
