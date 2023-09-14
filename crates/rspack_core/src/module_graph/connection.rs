use std::hash::Hash;

use crate::{DependencyCondition, DependencyId, ModuleGraph, ModuleIdentifier, RuntimeSpec};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConnectionId(usize);

impl std::ops::Deref for ConnectionId {
  type Target = usize;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<usize> for ConnectionId {
  fn from(id: usize) -> Self {
    Self(id)
  }
}

#[derive(Debug, Clone)]
pub struct ModuleGraphConnection {
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,

  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,

  /// The referencing dependency id
  pub dependency_id: DependencyId,
  active: bool,
  conditional: bool,
  condition: Option<DependencyCondition>,
}

/// implementing hash by hand because condition maybe a function, which can't be hash
impl Hash for ModuleGraphConnection {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.original_module_identifier.hash(state);
    self.module_identifier.hash(state);
    self.dependency_id.hash(state);
    self.active.hash(state);
    self.conditional.hash(state);
  }
}
/// implementing hash by hand because condition maybe a function, which can't be hash
impl PartialEq for ModuleGraphConnection {
  fn eq(&self, other: &Self) -> bool {
    self.original_module_identifier == other.original_module_identifier
      && self.module_identifier == other.module_identifier
      && self.dependency_id == other.dependency_id
      && self.active == other.active
      && self.conditional == other.conditional
  }
}

/// implementing eq
impl Eq for ModuleGraphConnection {
  fn assert_receiver_is_total_eq(&self) {}
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: DependencyId,
    module_identifier: ModuleIdentifier,
    condition: Option<DependencyCondition>,
  ) -> Self {
    let active = !matches!(condition, Some(DependencyCondition::False));
    let conditional = condition.is_some();
    Self {
      original_module_identifier,
      module_identifier,
      dependency_id,
      active,
      conditional,
      condition,
    }
  }

  pub fn set_active(&mut self, value: bool) {
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
    match self.condition.as_ref().expect("should have condition") {
      DependencyCondition::False => ConnectionState::Bool(false),
      DependencyCondition::Fn(f) => f(self, runtime, module_graph),
    }
  }
}

#[derive(Debug)]
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
}

pub fn add_connection_states(a: ConnectionState, b: ConnectionState) -> ConnectionState {
  if matches!(a, ConnectionState::Bool(true)) || matches!(b, ConnectionState::Bool(true)) {
    return ConnectionState::Bool(true);
  }
  if matches!(a, ConnectionState::Bool(false)) {
    return ConnectionState::Bool(false);
  }
  if matches!(b, ConnectionState::Bool(false)) {
    return ConnectionState::Bool(false);
  }
  if matches!(a, ConnectionState::TransitiveOnly) {
    return b;
  }
  if matches!(b, ConnectionState::TransitiveOnly) {
    return a;
  }
  a
}
