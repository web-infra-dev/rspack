use std::{
  collections::{HashMap, HashSet},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use crate::{BundleOptions, JsModule, ResolvedId};
use crossbeam::{
  channel::{self},
  queue::SegQueue,
};
use dashmap::DashSet;
use futures::future::join_all;
use petgraph::graph::NodeIndex;
use smol_str::SmolStr;
use tracing::instrument;

#[derive(Debug, Default)]
pub struct ModuleGraph {
  pub resolved_entries: Vec<ResolvedId>,
  pub id_to_node_idx: HashMap<SmolStr, NodeIndex>,
  // pub relation_graph: ModulePetGraph,
  pub ordered_modules: Vec<SmolStr>,
  pub module_by_id: HashMap<SmolStr, JsModule>,
}

impl ModuleGraph {
  pub fn node_idx_of_enties(&self) -> Vec<NodeIndex> {
    self
      .resolved_entries
      .iter()
      .map(|rid| *self.id_to_node_idx.get(&rid.id).unwrap())
      .collect()
  }

  pub fn sort_modules(&mut self) {
    let mut stack = self
      .resolved_entries
      .iter()
      .map(|rid| rid.id.clone())
      .rev()
      .collect::<Vec<_>>();
    let mut dyn_imports = vec![];
    let mut visited = HashSet::new();
    let mut next_exec_order = 0;
    while let Some(id) = stack.pop() {
      let module = self
        .module_by_id
        .get_mut(&id)
        .unwrap_or_else(|| panic!("get id: {} failed", &id.as_str()));
      if !visited.contains(&id) {
        visited.insert(id.clone());
        stack.push(id);
        module
          .dependencies
          .keys()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_ids.get(dep).unwrap().clone();
            stack.push(rid.id);
          });
        module
          .dyn_imports
          .iter()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_ids.get(&dep.argument).unwrap().clone();
            dyn_imports.push(rid.id);
          });
      } else {
        module.exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    stack = dyn_imports.into_iter().rev().collect();
    while let Some(id) = stack.pop() {
      let module = self.module_by_id.get_mut(&id).unwrap();
      if !visited.contains(&id) {
        visited.insert(id.clone());
        stack.push(id);
        module
          .dependencies
          .keys()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_ids.get(dep).unwrap().clone();
            stack.push(rid.id);
          });
      } else {
        module.exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    let mut modules = self.module_by_id.values().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.exec_order);
    tracing::trace!(
      "ordered {:#?}",
      modules.iter().map(|m| &m.id).collect::<Vec<_>>()
    );
    self.ordered_modules = modules.iter().map(|m| m.id.clone()).collect();
  }
}
