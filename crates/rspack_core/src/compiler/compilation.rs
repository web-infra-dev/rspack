use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use dashmap::DashSet;

use crate::{
  split_chunks::code_splitting2, ChunkGraph, CompilerOptions, Dependency, EntryItem, ModuleGraph,
  ResolveKind,
};

#[derive(Debug, Default)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: Arc<DashSet<String>>,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
}

impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: HashMap<String, EntryItem>,
    visited_module_id: Arc<DashSet<String>>,
    module_graph: ModuleGraph,
  ) -> Self {
    Self {
      options,
      visited_module_id,
      module_graph,
      entries,
      chunk_graph: Default::default(),
    }
  }

  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn entry_dependencies(&self) -> Vec<Dependency> {
    self
      .entries
      .iter()
      .map(|(_name, detail)| Dependency {
        importer: None,
        specifier: detail.path.clone(),
        kind: ResolveKind::Import,
      })
      .collect()
  }

  pub fn calc_exec_order(&mut self) {
    // let mut entries = self.entry_dependencies();
    let mut entries = self
      .entry_dependencies()
      .iter()
      .filter_map(|dep| self.module_graph.module_by_dependency(dep))
      .map(|module| module.uri.clone())
      .collect::<Vec<_>>();

    self
      .module_graph
      .modules()
      .flat_map(|module| module.dynamic_depended_modules(&self.module_graph))
      .for_each(|dyn_mod| {
        entries.push(dyn_mod.uri.to_string());
      });

    let mut visited = HashSet::new();

    let mut next_exec_order = 0;
    for entry in entries {
      let mut stack_visited: HashSet<String> = HashSet::new();
      let mut stack = vec![entry];
      while let Some(module_uri) = stack.pop() {
        if !visited.contains(&module_uri) {
          if stack_visited.contains(module_uri.as_str()) {
            self
              .module_graph
              .module_by_uri_mut(&module_uri)
              .unwrap()
              .exec_order = next_exec_order;
            tracing::debug!(
              "module: {:?},next_exec_order {:?}",
              module_uri,
              next_exec_order
            );
            next_exec_order += 1;
            visited.insert(module_uri);
          } else {
            stack.push(module_uri.to_string());
            stack_visited.insert(module_uri.to_string());
            stack.append(
              &mut self
                .module_graph
                .module_by_uri(&module_uri)
                .unwrap()
                .depended_modules(&self.module_graph)
                .into_iter()
                .rev()
                .map(|dep_mod| &dep_mod.uri)
                .cloned()
                .collect(),
            )
          }
        }
      }
    }

    let mut modules = self.module_graph.modules().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.exec_order);
    tracing::debug!(
      "ordered {:#?}",
      modules.iter().map(|m| &m.uri).collect::<Vec<_>>()
    );
  }

  pub fn seal(&mut self) {
    code_splitting2(self);

    // optmize chunks
  }
}
