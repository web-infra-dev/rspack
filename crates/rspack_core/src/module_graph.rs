use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use hashbrown::{HashMap, HashSet};

use rspack_error::{internal_error, Error, Result};

use crate::{BoxModule, BoxModuleDependency, ModuleGraphModule, ModuleIdentifier};

// FIXME: placing this as global id is not acceptable, move it to somewhere else later
static NEXT_MODULE_GRAPH_CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,
  /// The referencing dependency id
  pub dependency_id: usize,

  /// The unique id of this connection
  pub id: usize,
}

impl Hash for ModuleGraphConnection {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.original_module_identifier.hash(state);
    self.module_identifier.hash(state);
    self.dependency_id.hash(state);
  }
}

impl PartialEq for ModuleGraphConnection {
  fn eq(&self, other: &Self) -> bool {
    self.original_module_identifier == other.original_module_identifier
      && self.module_identifier == other.module_identifier
      && self.dependency_id == other.dependency_id
  }
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: usize,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      original_module_identifier,
      module_identifier,
      dependency_id,

      id: NEXT_MODULE_GRAPH_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
  dependency_id_to_module_identifier: HashMap<usize, ModuleIdentifier>,

  /// Module identifier to its module
  pub(crate) module_identifier_to_module: HashMap<ModuleIdentifier, BoxModule>,
  /// Module identifier to its module graph module
  pub(crate) module_identifier_to_module_graph_module: HashMap<ModuleIdentifier, ModuleGraphModule>,

  dependency_id_to_connection_id: HashMap<usize, usize>,
  connection_id_to_dependency_id: HashMap<usize, usize>,
  dependency_id_to_dependency: HashMap<usize, BoxModuleDependency>,
  dependency_to_dependency_id: HashMap<BoxModuleDependency, usize>,

  /// The module graph connections
  connections: HashSet<ModuleGraphConnection>,
  connection_id_to_connection: HashMap<usize, ModuleGraphConnection>,
}

impl ModuleGraph {
  pub fn add_module_graph_module(&mut self, module_graph_module: ModuleGraphModule) {
    if let hashbrown::hash_map::Entry::Vacant(val) = self
      .module_identifier_to_module_graph_module
      .entry(module_graph_module.module_identifier)
    {
      val.insert(module_graph_module);
    }
  }

  pub fn add_module(&mut self, module: BoxModule) {
    if let hashbrown::hash_map::Entry::Vacant(val) =
      self.module_identifier_to_module.entry(module.identifier())
    {
      val.insert(module);
    }
  }

  pub fn add_dependency(
    &mut self,
    dep: BoxModuleDependency,
    module_identifier: ModuleIdentifier,
  ) -> usize {
    static NEXT_DEPENDENCY_ID: AtomicUsize = AtomicUsize::new(0);

    let id = NEXT_DEPENDENCY_ID.fetch_add(1, Ordering::Relaxed);
    self.dependency_id_to_dependency.insert(id, dep.clone());
    self.dependency_to_dependency_id.insert(dep, id);

    self
      .dependency_id_to_module_identifier
      .insert(id, module_identifier);

    id
  }

  /// Uniquely identify a module by its dependency
  pub fn module_by_dependency(&self, dep: &BoxModuleDependency) -> Option<&ModuleGraphModule> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_module_identifier.get(id))
      .and_then(|module_identifier| {
        self
          .module_identifier_to_module_graph_module
          .get(module_identifier)
      })
  }

  /// Get the dependency id of a dependency
  pub fn dependency_id_by_dependency(&self, dep: &BoxModuleDependency) -> Option<usize> {
    self.dependency_to_dependency_id.get(dep).cloned()
  }

  /// Return an unordered iterator of module graph modules
  pub fn module_graph_modules(&self) -> impl Iterator<Item = &ModuleGraphModule> {
    self.module_identifier_to_module_graph_module.values()
  }

  /// Return an unordered iterator of modules
  pub fn modules(&self) -> impl Iterator<Item = &BoxModule> {
    self.module_identifier_to_module.values()
  }

  /// Add a connection between two module graph modules, if a connection exists, then it will be reused.
  pub fn set_resolved_module(
    &mut self,
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: usize,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    let new_connection =
      ModuleGraphConnection::new(original_module_identifier, dependency_id, module_identifier);

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

    self
      .connection_id_to_dependency_id
      .insert(connection_id, dependency_id);

    {
      let mgm = self
        .module_graph_module_by_identifier_mut(&module_identifier)
        .ok_or_else(|| {
          Error::InternalError(internal_error!(format!(
            "Failed to set resolved module: Module linked to module identifier {module_identifier} cannot be found"
          )))
        })?;

      mgm.add_incoming_connection(connection_id);
    }

    if let Some(identifier) = original_module_identifier && let Some(original_mgm) = self.
    module_graph_module_by_identifier_mut(&identifier) {
        original_mgm.add_outgoing_connection(connection_id);
    };

    Ok(())
  }

  /// Uniquely identify a module by its identifier and return the aliased reference
  #[inline]
  pub fn module_by_identifier(&self, identifier: &ModuleIdentifier) -> Option<&BoxModule> {
    self.module_identifier_to_module.get(identifier)
  }

  /// Uniquely identify a module by its identifier and return the exclusive reference
  #[inline]
  pub fn module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut BoxModule> {
    self.module_identifier_to_module.get_mut(identifier)
  }

  /// Uniquely identify a module graph module by its module's identifier and return the aliased reference
  #[inline]
  pub fn module_graph_module_by_identifier(
    &self,
    identifier: &ModuleIdentifier,
  ) -> Option<&ModuleGraphModule> {
    self
      .module_identifier_to_module_graph_module
      .get(identifier)
  }

  /// Uniquely identify a module graph module by its module's identifier and return the exclusive reference
  #[inline]
  pub fn module_graph_module_by_identifier_mut(
    &mut self,
    identifier: &ModuleIdentifier,
  ) -> Option<&mut ModuleGraphModule> {
    self
      .module_identifier_to_module_graph_module
      .get_mut(identifier)
  }

  /// Uniquely identify a connection by a given dependency
  pub fn connection_by_dependency(
    &self,
    dep: &BoxModuleDependency,
  ) -> Option<&ModuleGraphConnection> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_connection_id.get(id))
      .and_then(|id| self.connection_id_to_connection.get(id))
  }

  pub fn dependency_by_connection(
    &self,
    connection: &ModuleGraphConnection,
  ) -> Option<&BoxModuleDependency> {
    self.dependency_by_connection_id(connection.id)
  }

  pub fn dependency_by_connection_id(&self, connection_id: usize) -> Option<&BoxModuleDependency> {
    self
      .connection_id_to_dependency_id
      .get(&connection_id)
      .and_then(|id| self.dependency_id_to_dependency.get(id))
  }

  pub fn connection_by_connection_id(
    &self,
    connection_id: usize,
  ) -> Option<&ModuleGraphConnection> {
    self.connection_id_to_connection.get(&connection_id)
  }

  pub fn get_pre_order_index(&self, module_identifier: &ModuleIdentifier) -> Option<usize> {
    self
      .module_graph_module_by_identifier(module_identifier)
      .and_then(|mgm| mgm.pre_order_index)
  }

  pub fn get_issuer(&self, module: &BoxModule) -> Option<&BoxModule> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .and_then(|mgm| mgm.get_issuer().get_module(self))
  }

  pub fn get_outgoing_connections(&self, module: &BoxModule) -> HashSet<&ModuleGraphConnection> {
    self
      .module_graph_module_by_identifier(&module.identifier())
      .map(|mgm| {
        mgm
          .outgoing_connections
          .iter()
          .filter_map(|id| self.connection_by_connection_id(*id))
          .collect()
      })
      .unwrap_or_default()
  }
}
