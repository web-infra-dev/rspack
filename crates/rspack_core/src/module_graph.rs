use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::{
  mapref::one::{Ref, RefMut},
  DashMap, DashSet,
};

use rspack_error::{Error, Result};

use crate::{Dependency, ModuleGraphModule, ModuleIdentifier, NormalModule};

// FIXME: placing this as global id is not acceptable, move it to somewhere else later
static MODULE_GRAPH_CONNECTION_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static DEPENDENCY_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module_identifier: ModuleIdentifier,
  pub dependency_id: u32,

  pub id: u32,
}

impl std::hash::Hash for ModuleGraphConnection {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.original_module_identifier.hash(state);
    self.module_identifier.hash(state);
    self.dependency_id.hash(state);
  }
}

impl std::cmp::PartialEq for ModuleGraphConnection {
  fn eq(&self, other: &Self) -> bool {
    self.original_module_identifier == other.original_module_identifier
      && self.module_identifier == other.module_identifier
      && self.dependency_id == other.dependency_id
  }
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: u32,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      original_module_identifier,
      module_identifier,
      dependency_id,

      id: MODULE_GRAPH_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  dependency_id_to_module_identifier: DashMap<u32, String>,

  pub module_identifier_to_module: DashMap<ModuleIdentifier, NormalModule>,
  pub module_identifier_to_module_graph_module: DashMap<ModuleIdentifier, ModuleGraphModule>,

  dependency_id_to_connection_id: DashMap<u32, u32>,
  dependency_id_to_dependency: DashMap<u32, Dependency>,
  dependency_to_dependency_id: DashMap<Dependency, u32>,

  pub connections: DashSet<ModuleGraphConnection>,
  connection_id_to_connection: DashMap<u32, ModuleGraphConnection>,
}

impl ModuleGraph {
  pub fn add_module_graph_module(&self, module_graph_module: ModuleGraphModule) {
    if let dashmap::mapref::entry::Entry::Vacant(val) = self
      .module_identifier_to_module_graph_module
      .entry(module_graph_module.module_identifier.clone())
    {
      val.insert(module_graph_module);
    }
  }

  pub fn add_module(&self, module: NormalModule) {
    if let dashmap::mapref::entry::Entry::Vacant(val) =
      self.module_identifier_to_module.entry(module.identifier())
    {
      val.insert(module);
    }
  }

  pub fn add_dependency(&self, (dep, dependency_id): (Dependency, u32), resolved_uri: String) {
    self
      .dependency_id_to_dependency
      .insert(dependency_id, dep.clone());
    self.dependency_to_dependency_id.insert(dep, dependency_id);

    self
      .dependency_id_to_module_identifier
      .insert(dependency_id, resolved_uri);
  }

  pub fn module_by_dependency(
    &self,
    dep: &Dependency,
  ) -> Option<Ref<'_, String, ModuleGraphModule>> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_module_identifier.get(&*id))
      .and_then(|module_identifier| {
        self
          .module_identifier_to_module_graph_module
          .get(&*module_identifier)
      })
  }

  pub fn set_resolved_module(
    &self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: u32,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let new_connection = ModuleGraphConnection::new(
      original_module_identifier.clone(),
      dependency_id,
      module_identifier.clone(),
    );

    let connection_id = if let Some(connection) = self.connections.get(&new_connection) {
      connection.id
    } else {
      let id = new_connection.id;
      self.connections.insert(new_connection.clone());
      self.connection_id_to_connection.insert(id, new_connection);
      id
    };

    self
      .dependency_id_to_connection_id
      .insert(dependency_id, connection_id);

    {
      let mgm = self
        .module_graph_module_by_identifier(&module_identifier)
        .ok_or_else(|| {
          Error::InternalError(format!(
            "Failed to set resolved module: Module linked to module identifier {} cannot be found",
            module_identifier
          ))
        })?;

      mgm.add_incoming_connection(connection_id);
    }

    if let Some(identifier) = original_module_identifier && let Some(original_mgm) = self.
    module_graph_module_by_identifier(&identifier) {
        original_mgm.add_outgoing_connection(connection_id);
    };

    Ok(())
  }

  #[inline]
  pub fn module_by_uri(&self, uri: &str) -> Option<Ref<'_, String, ModuleGraphModule>> {
    self.module_identifier_to_module_graph_module.get(uri)
  }

  #[inline]
  pub fn module_by_identifier(&self, identifier: &str) -> Option<Ref<'_, String, NormalModule>> {
    self.module_identifier_to_module.get(identifier)
  }

  #[inline]
  pub fn module_by_identifier_mut(
    &self,
    identifier: &str,
  ) -> Option<RefMut<'_, String, NormalModule>> {
    self.module_identifier_to_module.get_mut(identifier)
  }

  #[inline]
  pub fn module_graph_module_by_identifier(
    &self,
    identifier: &str,
  ) -> Option<Ref<'_, String, ModuleGraphModule>> {
    self
      .module_identifier_to_module_graph_module
      .get(identifier)
  }

  #[inline]
  pub fn module_graph_module_by_identifier_mut(
    &self,
    identifier: &str,
  ) -> Option<RefMut<'_, String, ModuleGraphModule>> {
    self
      .module_identifier_to_module_graph_module
      .get_mut(identifier)
  }

  #[inline]
  pub fn module_by_uri_mut(&self, uri: &str) -> Option<RefMut<'_, String, ModuleGraphModule>> {
    self.module_identifier_to_module_graph_module.get_mut(uri)
  }

  pub fn connection_by_dependency(
    &self,
    dep: &Dependency,
  ) -> Option<Ref<'_, u32, ModuleGraphConnection>> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_connection_id.get(&*id))
      .and_then(|id| self.connection_id_to_connection.get(&*id))
  }

  pub fn connection_by_connection_id(
    &self,
    connection_id: u32,
  ) -> Option<Ref<'_, u32, ModuleGraphConnection>> {
    self.connection_id_to_connection.get(&connection_id)
  }
}
