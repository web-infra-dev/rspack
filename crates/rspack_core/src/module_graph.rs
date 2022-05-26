use std::collections::{HashMap, HashSet};

use crate::{JsModule, ResolvedURI};
use petgraph::graph::NodeIndex;
use tracing::instrument;

#[derive(Debug, Default)]
pub struct ModuleGraphContainer {
  pub resolved_entries: HashMap<String, ResolvedURI>,
  pub ordered_modules: Vec<String>,
  pub module_graph: ModuleGraph,
}

impl ModuleGraphContainer {
  #[instrument(skip(self))]
  pub fn sort_modules(&mut self) {
    let mut stack = self
      .resolved_entries
      .values()
      .map(|rid| rid.uri.clone())
      .collect::<Vec<_>>()
      .into_iter()
      .rev()
      .collect::<Vec<_>>();
    let mut dyn_imports = vec![];
    let mut visited = HashSet::new();
    let mut next_exec_order = 0;
    while let Some(uri) = stack.pop() {
      let module = self.module_graph.module_by_uri_mut(&uri).unwrap();
      if !visited.contains(&uri) {
        visited.insert(uri.clone());
        stack.push(uri);
        module
          .dependencies
          .keys()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_uris.get(dep).unwrap().clone();
            stack.push(rid.uri);
          });
        module
          .dyn_imports
          .iter()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_uris.get(&dep.argument).unwrap().clone();
            dyn_imports.push(rid.uri);
          });
        module.exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    stack = dyn_imports.into_iter().rev().collect();
    while let Some(uri) = stack.pop() {
      let module = self.module_graph.module_by_uri_mut(&uri).unwrap();
      if !visited.contains(&uri) {
        visited.insert(uri.clone());
        stack.push(uri);
        module
          .dependencies
          .keys()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_uris.get(dep).unwrap().clone();
            stack.push(rid.uri);
          });
        module.exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    let mut modules = self.module_graph.modules().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.exec_order);
    tracing::trace!(
      "ordered {:#?}",
      modules.iter().map(|m| &m.uri).collect::<Vec<_>>()
    );
    self.ordered_modules = modules.iter().map(|m| m.uri.clone()).collect();
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  uri_to_module: hashbrown::HashMap<String, JsModule>,
  id_to_uri: hashbrown::HashMap<String, String>,
}

impl ModuleGraph {
  pub fn add_module(&mut self, module: JsModule) {
    let uri = module.uri.clone();
    let id = module.id.clone();
    self.uri_to_module.insert(uri.clone(), module);
    self.id_to_uri.insert(id, uri);
  }

  pub fn remove_by_uri(&mut self, uri: &str) -> Option<JsModule> {
    let js_mod = self.uri_to_module.remove(uri)?;
    self.id_to_uri.remove(&js_mod.id);
    Some(js_mod)
  }

  pub fn remove_by_id(&mut self, id: &str) -> Option<JsModule> {
    let uri = self.id_to_uri.get(id)?;
    let js_mod = self.uri_to_module.remove(uri)?;
    self.id_to_uri.remove(id);
    Some(js_mod)
  }

  #[inline]
  pub fn module_by_uri(&self, uri: &str) -> Option<&JsModule> {
    self.uri_to_module.get(uri)
    // .unwrap_or_else(|| panic!("fail to find module by uri: {:?}", uri))
  }

  #[inline]
  pub fn module_by_uri_mut(&mut self, uri: &str) -> Option<&mut JsModule> {
    self.uri_to_module.get_mut(uri)
    // .unwrap_or_else(|| panic!("fail to find module by uri: {:?}", uri))
  }

  #[inline]
  pub fn module_by_id(&self, id: &str) -> Option<&JsModule> {
    self.uri_to_module.get(&self.id_to_uri[id])
    // .unwrap_or_else(|| panic!("fail to find module by id: {:?}", id))
  }

  #[inline]
  pub fn module_by_id_mut(&mut self, id: &str) -> Option<&mut JsModule> {
    self.uri_to_module.get_mut(&self.id_to_uri[id])
    // .unwrap_or_else(|| panic!("fail to find module by id: {:?}", id))
  }

  pub fn modules(&self) -> impl Iterator<Item = &JsModule> {
    self.uri_to_module.values()
  }

  pub fn ids(&self) -> impl Iterator<Item = &String> {
    self.id_to_uri.keys()
  }

  pub fn uris(&self) -> impl Iterator<Item = &String> {
    self.uri_to_module.keys()
  }
}
