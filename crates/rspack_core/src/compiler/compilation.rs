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
    let mut stack = self
      .entry_dependencies()
      .iter()
      .filter_map(|dep| self.module_graph.module_by_dependency(dep))
      .map(|module| module.uri.clone())
      .collect::<Vec<_>>();

    let mut dyn_imports = vec![];
    let mut visited: HashSet<String> = HashSet::new();
    let mut next_exec_order = 0;
    while let Some(uri) = stack.pop() {
      let module = self.module_graph.module_by_uri(&uri).unwrap();
      if !visited.contains(&uri) {
        visited.insert(uri.clone());
        stack.push(uri.clone());
        module
          .depended_modules(&self.module_graph)
          .into_iter()
          .rev()
          .for_each(|dep| {
            stack.push(dep.uri.clone());
          });
        module
          .dynamic_depended_modules(&self.module_graph)
          .into_iter()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            dyn_imports.push(dep.uri.clone());
          });

        self
          .module_graph
          .module_by_uri_mut(&uri)
          .unwrap()
          .exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    stack = dyn_imports.into_iter().rev().collect();
    while let Some(uri) = stack.pop() {
      let module = self.module_graph.module_by_uri(&uri).unwrap();
      if !visited.contains(&uri) {
        visited.insert(uri.clone());
        stack.push(uri.clone());
        module
          .depended_modules(&self.module_graph)
          .into_iter()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            stack.push(dep.uri.to_string());
          });
        self
          .module_graph
          .module_by_uri_mut(&uri)
          .unwrap()
          .exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    let mut modules = self.module_graph.modules().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.exec_order);
    tracing::trace!(
      "ordered {:#?}",
      modules.iter().map(|m| &m.uri).collect::<Vec<_>>()
    );
  }

  pub fn seal(&mut self) {
    code_splitting2(self);

    // optmize chunks
  }
}
