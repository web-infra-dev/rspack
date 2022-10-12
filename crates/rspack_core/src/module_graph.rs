use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::{
  mapref::{
    multiple::RefMulti,
    one::{Ref, RefMut},
  },
  DashMap,
};
use hashbrown::HashMap;

use rspack_error::{Error, Result};

use crate::{Dependency, ModuleDependency, ModuleGraphModule, ModuleIdentifier, NormalModule};

static MODULE_GRAPH_CONNECTION_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleGraphConnection {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module_identifier: ModuleIdentifier,
  pub dependency: ModuleDependency,

  pub id: u32,
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency: ModuleDependency,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      original_module_identifier,
      module_identifier,
      dependency,

      id: MODULE_GRAPH_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  uri_to_module: DashMap<String, ModuleGraphModule>,
  dependency_to_module_uri: DashMap<Dependency, String>,

  // TODO: cleanup `pub`
  pub module_identifier_to_module: DashMap<ModuleIdentifier, NormalModule>,
  pub module_identifier_to_module_graph_module: DashMap<ModuleIdentifier, ModuleGraphModule>,

  pub connections: DashMap<u32, ModuleGraphConnection>,
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

  pub fn set_resolved_module(
    &self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency: ModuleDependency,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let connection = ModuleGraphConnection::new(
      original_module_identifier.clone(),
      dependency,
      module_identifier.clone(),
    );
    let connection_id = connection.id;
    self.connections.insert(connection_id, connection);

    let mut mgm = self
      .module_identifier_to_module_graph_module
      .get_mut(&module_identifier)
      .ok_or_else(|| {
        Error::InternalError(format!(
          "Failed to set resolved module: Module linked to module identifier {} cannot be found",
          module_identifier
        ))
      })?;

    mgm.add_incoming_connection(connection_id);

    if let Some(identifier) = original_module_identifier && let Some(mut original_mgm) = self
    .module_identifier_to_module_graph_module
    .get_mut(&identifier) {
        original_mgm.add_outgoing_connection(connection_id);
    };

    Ok(())
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
