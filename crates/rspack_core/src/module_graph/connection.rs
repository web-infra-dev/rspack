use std::hash::Hash;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

use crate::{DependencyCondition, DependencyId, ModuleGraph, ModuleIdentifier, RuntimeSpec};

pub static CONNECTION_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConnectionId(u32);

impl ConnectionId {
  pub fn new() -> Self {
    Self(CONNECTION_ID.fetch_add(1, Relaxed))
  }
}

impl Default for ConnectionId {
  fn default() -> Self {
    Self::new()
  }
}
impl std::ops::Deref for ConnectionId {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl From<u32> for ConnectionId {
  fn from(id: u32) -> Self {
    Self(id)
  }
}

#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  pub id: ConnectionId,
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub resolved_original_module_identifier: Option<ModuleIdentifier>,

  /// The referenced module identifier
  module_identifier: ModuleIdentifier,

  /// The referencing dependency id
  pub dependency_id: DependencyId,
  pub active: bool,
  pub conditional: bool,
}

impl Hash for ModuleGraphConnection {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}

impl PartialEq for ModuleGraphConnection {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: DependencyId,
    module_identifier: ModuleIdentifier,
    active: bool,
    conditional: bool,
  ) -> Self {
    Self {
      id: ConnectionId::new(),
      original_module_identifier,
      module_identifier,
      dependency_id,
      active,
      conditional,
      resolved_original_module_identifier: original_module_identifier,
    }
  }

  pub fn set_active(&mut self, value: bool) {
    self.conditional = false;
    self.active = value;
  }

  pub fn is_active(&self, module_graph: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> bool {
    if !self.conditional {
      return self.active;
    }
    self
      .get_condition_state(module_graph, runtime)
      .is_not_false()
  }

  pub fn is_target_active(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if !self.conditional {
      return self.active;
    }
    self.get_condition_state(module_graph, runtime).is_true()
  }

  pub fn get_active_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> ConnectionState {
    if !self.conditional {
      return ConnectionState::Bool(self.active);
    }

    self.get_condition_state(module_graph, runtime)
  }

  /// ## Panic
  /// This function will panic if we don't have condition, make sure you checked if `condition`
  /// exists before you invoke this function
  /// Here avoid move condition, so use dependency id to search
  pub fn get_condition_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> ConnectionState {
    match module_graph
      .connection_to_condition
      .get(&self.id)
      .unwrap_or_else(|| panic!("{:#?}", self))
    {
      DependencyCondition::False => ConnectionState::Bool(false),
      DependencyCondition::Fn(f) => f(self, runtime, module_graph),
    }
  }

  pub fn module_identifier(&self) -> &ModuleIdentifier {
    &self.module_identifier
  }

  /// used for set module identifier after clone the [ModuleGraphConnection]
  pub fn set_module_identifier(&mut self, mi: ModuleIdentifier, mg: &mut ModuleGraph) {
    self.module_identifier = mi;
    mg.dependency_id_to_module_identifier
      .insert(self.dependency_id, mi);
  }

  /// used for mutate module identifier, don't forget also set the module identifier of the
  /// related dependency_id
  pub fn set_module_identifier_only(&mut self, mi: ModuleIdentifier) {
    self.module_identifier = mi;
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionState {
  Bool(bool),
  CircularConnection,
  TransitiveOnly,
}

impl ConnectionState {
  pub fn is_true(&self) -> bool {
    matches!(self, ConnectionState::Bool(true))
  }

  pub fn is_not_false(&self) -> bool {
    !matches!(self, ConnectionState::Bool(false))
  }

  pub fn is_false(&self) -> bool {
    !self.is_not_false()
  }
}

pub fn add_connection_states(a: ConnectionState, b: ConnectionState) -> ConnectionState {
  if matches!(a, ConnectionState::Bool(true)) || matches!(b, ConnectionState::Bool(true)) {
    return ConnectionState::Bool(true);
  }
  if matches!(a, ConnectionState::Bool(false)) {
    return b;
  }
  if matches!(b, ConnectionState::Bool(false)) {
    return a;
  }
  if matches!(a, ConnectionState::TransitiveOnly) {
    return b;
  }
  if matches!(b, ConnectionState::TransitiveOnly) {
    return a;
  }
  a
}
