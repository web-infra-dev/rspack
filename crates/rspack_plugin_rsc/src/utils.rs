use std::collections::VecDeque;

use rspack_collections::IdentifierSet;
use rspack_core::{
  AsyncDependenciesBlockIdentifier, Compilation, ConnectionState, DependenciesBlock, GroupOptions,
  Module, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphRef, ModuleIdentifier, NormalModule,
  RSCModuleType, RuntimeSpec, get_entry_runtime,
};
use rspack_util::queue::Queue;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum DependenciesBlockIdentifier {
  Module(ModuleIdentifier),
  AsyncDependenciesBlock(AsyncDependenciesBlockIdentifier),
}

pub struct ServerEntries<'a> {
  module_graph: &'a ModuleGraph<'a>,
  module_graph_cache: &'a ModuleGraphCacheArtifact,
  entries_queue: Queue<(DependenciesBlockIdentifier, Option<RuntimeSpec>)>,
  runtime: Option<RuntimeSpec>,
  blocks_queue: VecDeque<DependenciesBlockIdentifier>,
  visited_modules: IdentifierSet,
}

impl<'a> ServerEntries<'a> {
  pub fn new(compilation: &'a Compilation, module_graph: &'a ModuleGraph<'a>) -> Self {
    let entries = &compilation.entries;

    let mut entries_queue: Queue<(DependenciesBlockIdentifier, Option<RuntimeSpec>)> =
      Default::default();
    for (entry_name, entry_data) in entries {
      let runtime = get_entry_runtime(entry_name, &entry_data.options, &entries);

      for &dependency_id in entry_data
        .dependencies
        .iter()
        .chain(entry_data.include_dependencies.iter())
      {
        let Some(module_identifier) =
          module_graph.module_identifier_by_dependency_id(&dependency_id)
        else {
          continue;
        };
        entries_queue.enqueue((
          DependenciesBlockIdentifier::Module(*module_identifier),
          Some(runtime.clone()),
        ));
      }
    }

    Self {
      module_graph,
      module_graph_cache: &compilation.module_graph_cache_artifact,
      entries_queue,
      runtime: None,
      blocks_queue: Default::default(),
      visited_modules: Default::default(),
    }
  }
}

impl<'a> Iterator for ServerEntries<'a> {
  type Item = (&'a NormalModule, Option<RuntimeSpec>);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(dependencies_block_identifier) = self.blocks_queue.pop_front() {
        let (async_dependencies_blocks, dependencies) = match dependencies_block_identifier {
          DependenciesBlockIdentifier::Module(module_identifier) => {
            if !self.visited_modules.insert(module_identifier) {
              continue;
            }
            let Some(module) = self.module_graph.module_by_identifier(&module_identifier) else {
              continue;
            };
            if let Some(normal_module) = module.as_normal_module() {
              if let Some(rsc) = &normal_module.build_info().rsc
                && rsc.module_type == RSCModuleType::ServerEntry
              {
                return Some((normal_module, self.runtime.clone()));
              }
            }
            (module.get_blocks(), module.get_dependencies())
          }
          DependenciesBlockIdentifier::AsyncDependenciesBlock(async_dependencies_block_id) => {
            let Some(async_dependencies_block) =
              self.module_graph.block_by_id(&async_dependencies_block_id)
            else {
              continue;
            };
            (
              async_dependencies_block.get_blocks(),
              async_dependencies_block.get_dependencies(),
            )
          }
        };

        for identifier in async_dependencies_blocks {
          let Some(async_dependencies_block) = self.module_graph.block_by_id(identifier) else {
            continue;
          };
          if let Some(GroupOptions::Entrypoint(entry_options)) =
            async_dependencies_block.get_group_options()
          {
            let runtime = RuntimeSpec::from_entry_options(entry_options);
            self.entries_queue.enqueue((
              DependenciesBlockIdentifier::AsyncDependenciesBlock(*identifier),
              runtime,
            ));
          } else {
            self
              .blocks_queue
              .push_back(DependenciesBlockIdentifier::AsyncDependenciesBlock(
                *identifier,
              ));
          }
        }

        for dependency_id in dependencies {
          let Some(connection) = self
            .module_graph
            .connection_by_dependency_id(&dependency_id)
          else {
            continue;
          };
          let active_state = connection.active_state(
            self.module_graph,
            self.runtime.as_ref(),
            self.module_graph_cache,
          );

          if active_state == ConnectionState::Active(false) {
            continue;
          }
          self
            .blocks_queue
            .push_back(DependenciesBlockIdentifier::Module(
              *connection.module_identifier(),
            ));
        }
      }

      if self.blocks_queue.is_empty() {
        if let Some((dependencies_block_identifier, runtime)) = self.entries_queue.dequeue() {
          self.runtime = runtime;
          self.blocks_queue.push_back(dependencies_block_identifier);
          self.visited_modules.clear();
        } else {
          break;
        }
      }
    }

    None
  }
}

pub struct EntryModules<'a> {
  entries_iter: indexmap::map::Iter<'a, String, rspack_core::EntryData>,
  module_graph: &'a ModuleGraphRef<'a>,
  // runtime: Option<RuntimeSpec>,
}

impl<'a> EntryModules<'a> {
  pub fn new(compilation: &'a Compilation, module_graph: &'a ModuleGraphRef<'a>) -> Self {
    let entries_iter = compilation.entries.iter();
    Self {
      entries_iter,
      module_graph,
    }
  }
}

impl<'a> Iterator for EntryModules<'a> {
  type Item = (&'a NormalModule, &'a str, Option<RuntimeSpec>);

  fn next(&mut self) -> Option<Self::Item> {
    if let Some((entry_name, entry_data)) = self.entries_iter.next() {
      let entry_dependency = entry_data.dependencies[0];
      if let Some(entry_module_identifier) =
        self.module_graph.get_resolved_module(&entry_dependency)
      {
        if let Some(entry_module) = self
          .module_graph
          .module_by_identifier(entry_module_identifier)
        {
          if let Some(normal_module) = entry_module.as_normal_module() {
            let runtime = RuntimeSpec::from_entry_options(&entry_data.options);
            return Some((normal_module, entry_name, runtime));
          }
        }
      }
    }
    None
  }
}
