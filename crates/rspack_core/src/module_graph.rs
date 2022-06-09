use std::collections::HashMap;

use crate::{Dependency, ModuleGraphModule};

#[derive(Debug, Default)]
pub struct ModuleGraph {
  uri_to_module: HashMap<String, ModuleGraphModule>,
  dependency_to_module_uri: HashMap<Dependency, String>,
  // id_to_uri: hashbrown::HashMap<String, String>,
}

impl ModuleGraph {
  pub fn add_module(&mut self, module: ModuleGraphModule) {
    // let id = module.id().to_owned();
    self.uri_to_module.insert(module.uri.clone(), module);
    // self.id_to_uri.insert(id, uri);
  }

  pub fn add_dependency(&mut self, dep: Dependency, resolved_uri: String) {
    self.dependency_to_module_uri.insert(dep, resolved_uri);
  }

  pub fn module_by_dependency(&self, dep: &Dependency) -> Option<&ModuleGraphModule> {
    let uri = self.dependency_to_module_uri.get(dep)?;
    self.uri_to_module.get(uri)
  }

  pub fn uri_by_dependency(&self, dep: &Dependency) -> Option<&str> {
    let uri = self.dependency_to_module_uri.get(dep)?;
    Some(uri.as_str())
  }

  pub fn module_by_dependency_mut(&mut self, dep: &Dependency) -> Option<&mut ModuleGraphModule> {
    let uri = self.dependency_to_module_uri.get(dep)?;
    self.uri_to_module.get_mut(uri)
  }

  pub fn modules(&self) -> impl Iterator<Item = &ModuleGraphModule> {
    self.uri_to_module.values()
  }

  // pub fn remove_by_uri(&mut self, uri: &str) -> Option<ModuleGraphModule> {
  //   let js_mod = self.uri_to_module.remove(uri)?;
  //   // self.id_to_uri.remove(&js_mod.id);
  //   Some(js_mod)
  // }

  // pub fn remove_by_id(&mut self, id: &str) -> Option<Module> {
  //   let uri = self.id_to_uri.get(id)?;
  //   let js_mod = self.uri_to_module.remove(uri)?;
  //   self.id_to_uri.remove(id);
  //   Some(js_mod)
  // }

  #[inline]
  pub fn module_by_uri(&self, uri: &str) -> Option<&ModuleGraphModule> {
    self.uri_to_module.get(uri)
    // .unwrap_or_else(|| panic!("fail to find module by uri: {:?}", uri))
  }

  #[inline]
  pub fn module_by_uri_mut(&mut self, uri: &str) -> Option<&mut ModuleGraphModule> {
    self.uri_to_module.get_mut(uri)
  }

  // #[inline]
  // pub fn module_by_id(&self, id: &str) -> Option<&Module> {
  //   self.uri_to_module.get(&self.id_to_uri[id])
  //   // .unwrap_or_else(|| panic!("fail to find module by id: {:?}", id))
  // }

  // #[inline]
  // pub fn module_by_id_mut(&mut self, id: &str) -> Option<&mut Module> {
  //   self.uri_to_module.get_mut(&self.id_to_uri[id])
  //   // .unwrap_or_else(|| panic!("fail to find module by id: {:?}", id))
  // }

  // pub fn modules(&self) -> impl Iterator<Item = &Module> {
  //   self.uri_to_module.values()
  // }

  // pub fn ids(&self) -> impl Iterator<Item = &String> {
  //   self.id_to_uri.keys()
  // }

  // pub fn uris(&self) -> impl Iterator<Item = &String> {
  //   self.uri_to_module.keys()
  // }
}
