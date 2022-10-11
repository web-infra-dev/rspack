use hashbrown::HashMap;

use crate::{Dependency, ModuleDependency, ModuleGraphModule, ModuleIdentifier, NormalModule};

#[derive(Debug, Hash)]
pub struct ModuleGraphConnection {
  original_module: Option<ModuleIdentifier>,
  module: ModuleIdentifier,
  dependency: ModuleDependency,
}

impl ModuleGraphConnection {
  pub fn new(
    original_module: Option<ModuleIdentifier>,
    module: ModuleIdentifier,
    dependency: ModuleDependency,
  ) -> Self {
    Self {
      original_module,
      module,
      dependency,
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  uri_to_module: HashMap<String, ModuleGraphModule>,
  dependency_to_module_uri: HashMap<Dependency, String>,

  module_identifier_to_module: HashMap<ModuleIdentifier, NormalModule>,
  module_identifier_to_module_graph_module: HashMap<ModuleIdentifier, ModuleGraphModule>,
  /* id_to_uri: hashbrown::HashMap<String, String>, */
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

  #[inline]
  pub fn module_by_uri(&self, uri: &str) -> Option<&ModuleGraphModule> {
    self.uri_to_module.get(uri)
    // .unwrap_or_else(|| panic!("fail to find module by uri: {:?}", uri))
  }

  #[inline]
  pub fn module_by_uri_mut(&mut self, uri: &str) -> Option<&mut ModuleGraphModule> {
    self.uri_to_module.get_mut(uri)
  }
}
