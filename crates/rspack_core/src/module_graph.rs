use dashmap::{
  mapref::{
    multiple::RefMulti,
    one::{Ref, RefMut},
  },
  DashMap,
};
use hashbrown::HashMap;

use crate::{Dependency, ModuleDependency, ModuleGraphModule, ModuleIdentifier, NormalModule};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ModuleGraphConnection {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module_identifier: ModuleIdentifier,
  pub dependency: ModuleDependency,
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    module_identifier: ModuleIdentifier,
    dependency: ModuleDependency,
  ) -> Self {
    Self {
      original_module_identifier,
      module_identifier,
      dependency,
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  uri_to_module: DashMap<String, ModuleGraphModule>,
  dependency_to_module_uri: DashMap<Dependency, String>,

  module_identifier_to_module: DashMap<ModuleIdentifier, NormalModule>,
  module_identifier_to_module_graph_module: DashMap<ModuleIdentifier, ModuleGraphModule>,
  /* id_to_uri: hashbrown::HashMap<String, String>, */
}

impl ModuleGraph {
  pub fn add_module_graph_module(&self, module_graph_module: ModuleGraphModule) {
    // self.uri_to_module.insert(
    //   module_graph_module.module_identifier.clone(),
    //   module_graph_module,
    // );
    self.module_identifier_to_module_graph_module.insert(
      module_graph_module.module_identifier.clone(),
      module_graph_module,
    );
  }

  pub fn add_module(&self, module: NormalModule) {
    // let id = module.id().to_owned();
    // self.uri_to_module.insert(module.identifier(), module);

    if let dashmap::mapref::entry::Entry::Vacant(val) =
      self.module_identifier_to_module.entry(module.identifier())
    {
      val.insert(module);
    }

    // self
    //   .module_identifier_to_module
    //   .insert(module.identifier(), module)
    // self.id_to_uri.insert(id, uri);
  }

  pub fn add_dependency(&self, dep: Dependency, resolved_uri: String) {
    self.dependency_to_module_uri.insert(dep, resolved_uri);
  }

  pub fn module_by_dependency(
    &self,
    dep: &Dependency,
  ) -> Option<Ref<'_, String, ModuleGraphModule>> {
    let uri = self.dependency_to_module_uri.get(dep)?;
    self.uri_to_module.get(&*uri)
  }

  // pub fn uri_by_dependency(&self, dep: &Dependency) -> Option<&str> {
  //   let uri = self.dependency_to_module_uri.get(dep)?;
  //   Some(uri.as_str())
  // }

  pub fn module_by_dependency_mut(
    &mut self,
    dep: &Dependency,
  ) -> Option<RefMut<'_, String, ModuleGraphModule>> {
    let uri = self.dependency_to_module_uri.get(dep)?;
    self.uri_to_module.get_mut(&*uri)
  }

  pub fn modules(
    &self,
  ) -> impl Iterator<Item = RefMulti<'_, std::string::String, ModuleGraphModule>> {
    self.uri_to_module.iter()
    // self.uri_to_module.values()
  }

  #[inline]
  pub fn module_by_uri(&self, uri: &str) -> Option<Ref<'_, String, ModuleGraphModule>> {
    self.uri_to_module.get(uri)
    // .unwrap_or_else(|| panic!("fail to find module by uri: {:?}", uri))
  }

  #[inline]
  pub fn module_by_identifier(&self, identifier: &str) -> Option<Ref<'_, String, NormalModule>> {
    self.module_identifier_to_module.get(identifier)
  }

  #[inline]
  pub fn module_by_uri_mut(&self, uri: &str) -> Option<RefMut<'_, String, ModuleGraphModule>> {
    self.uri_to_module.get_mut(uri)
  }
}
