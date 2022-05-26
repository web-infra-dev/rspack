use std::collections::{HashMap, HashSet};

use crate::{JsModule, ResolvedURI};
use petgraph::graph::NodeIndex;
use tracing::instrument;

#[derive(Debug, Default)]
pub struct ModuleGraph {
  pub resolved_entries: HashMap<String, ResolvedURI>,
  pub ordered_modules: Vec<String>,
  pub module_by_id: HashMap<String, JsModule>,
}

impl ModuleGraph {
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
    while let Some(id) = stack.pop() {
      if let Some(module) = self.module_by_id.get_mut(&id) {
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
              let rid = module.resolved_uris.get(dep).unwrap().clone();
              stack.push(rid.uri);
            });
          module.exec_order = next_exec_order;
          next_exec_order += 1;
        }
      }
    }
    let mut modules = self.module_by_id.values().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.exec_order);
    tracing::trace!(
      "ordered {:#?}",
      modules.iter().map(|m| &m.uri).collect::<Vec<_>>()
    );
    self.ordered_modules = modules.iter().map(|m| m.uri.clone()).collect();
  }
}

#[derive(Debug, Default)]
pub struct ModGraph {
  uri_to_module: hashbrown::HashMap<String, JsModule>,
  id_to_uri: hashbrown::HashMap<String, String>,
}

impl ModGraph {
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
  }

  #[inline]
  pub fn module_by_uri_mut(&mut self, uri: &str) -> Option<&mut JsModule> {
    self.uri_to_module.get_mut(uri)
  }

  #[inline]
  pub fn module_by_id(&self, id: &str) -> Option<&JsModule> {
    self.uri_to_module.get(&self.id_to_uri[id])
  }

  #[inline]
  pub fn module_by_id_mut(&mut self, id: &str) -> Option<&mut JsModule> {
    self.uri_to_module.get_mut(&self.id_to_uri[id])
  }
}
