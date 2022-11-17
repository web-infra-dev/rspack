use std::sync::atomic::{AtomicU32, Ordering};

use hashbrown::{HashMap, HashSet};

use rspack_error::{Error, Result};

use crate::{BoxModule, Dependency, ModuleGraphModule, ModuleIdentifier};

// FIXME: placing this as global id is not acceptable, move it to somewhere else later
static MODULE_GRAPH_CONNECTION_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static DEPENDENCY_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,
  /// The referencing dependency id
  pub dependency_id: u32,

  /// The unique id of this connection
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
  dependency_id_to_module_identifier: HashMap<u32, String>,

  /// Module identifier to its module
  pub module_identifier_to_module: HashMap<ModuleIdentifier, BoxModule>,
  /// Module identifier to its module graph module
  pub module_identifier_to_module_graph_module: HashMap<ModuleIdentifier, ModuleGraphModule>,

  dependency_id_to_connection_id: HashMap<u32, u32>,
  dependency_id_to_dependency: HashMap<u32, Dependency>,
  dependency_to_dependency_id: HashMap<Dependency, u32>,

  /// The module graph connections
  pub connections: HashSet<ModuleGraphConnection>,
  connection_id_to_connection: HashMap<u32, ModuleGraphConnection>,
}

impl ModuleGraph {
  pub fn add_module_graph_module(&mut self, module_graph_module: ModuleGraphModule) {
    if let hashbrown::hash_map::Entry::Vacant(val) = self
      .module_identifier_to_module_graph_module
      .entry(module_graph_module.module_identifier.clone())
    {
      val.insert(module_graph_module);
    }
  }

  pub fn add_module(&mut self, module: BoxModule) {
    if let hashbrown::hash_map::Entry::Vacant(val) = self
      .module_identifier_to_module
      .entry(module.identifier().into())
    {
      val.insert(module);
    }
  }

  pub fn add_dependency(
    &mut self,
    (dep, dependency_id): (Dependency, u32),
    module_identifier: String,
  ) {
    self
      .dependency_id_to_dependency
      .insert(dependency_id, dep.clone());
    self.dependency_to_dependency_id.insert(dep, dependency_id);

    self
      .dependency_id_to_module_identifier
      .insert(dependency_id, module_identifier);
  }

  /// Uniquely identify a module by its dependency
  pub fn module_by_dependency(&self, dep: &Dependency) -> Option<&ModuleGraphModule> {
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
  pub fn dependency_id_by_dependency(&self, dep: &Dependency) -> Option<u32> {
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
        .module_graph_module_by_identifier_mut(&module_identifier)
        .ok_or_else(|| {
          Error::InternalError(format!(
            "Failed to set resolved module: Module linked to module identifier {} cannot be found",
            module_identifier
          ))
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
  pub fn module_by_identifier(&self, identifier: &str) -> Option<&BoxModule> {
    self.module_identifier_to_module.get(identifier)
  }

  /// Uniquely identify a module by its identifier and return the exclusive reference
  #[inline]
  pub fn module_by_identifier_mut(&mut self, identifier: &str) -> Option<&mut BoxModule> {
    self.module_identifier_to_module.get_mut(identifier)
  }

  /// Uniquely identify a module graph module by its module's identifier and return the aliased reference
  #[inline]
  pub fn module_graph_module_by_identifier(&self, identifier: &str) -> Option<&ModuleGraphModule> {
    self
      .module_identifier_to_module_graph_module
      .get(identifier)
  }

  /// Uniquely identify a module graph module by its module's identifier and return the exclusive reference
  #[inline]
  pub fn module_graph_module_by_identifier_mut(
    &mut self,
    identifier: &str,
  ) -> Option<&mut ModuleGraphModule> {
    self
      .module_identifier_to_module_graph_module
      .get_mut(identifier)
  }

  /// Uniquely identify a connection by a given dependency
  pub fn connection_by_dependency(&self, dep: &Dependency) -> Option<&ModuleGraphConnection> {
    self
      .dependency_to_dependency_id
      .get(dep)
      .and_then(|id| self.dependency_id_to_connection_id.get(id))
      .and_then(|id| self.connection_id_to_connection.get(id))
  }

  pub fn connection_by_connection_id(&self, connection_id: u32) -> Option<&ModuleGraphConnection> {
    self.connection_id_to_connection.get(&connection_id)
  }
}
