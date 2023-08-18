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
// ,

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ModuleGraphConnection {
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,

  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,

  /// The referencing dependency id
  pub dependency_id: DependencyId,
  active: bool,
  conditional: bool,
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
      original_module_identifier,
      module_identifier,
      dependency_id,
      active,
      conditional,
    }
  }

  pub fn set_active(&mut self, value: bool) {
    self.active = value;
  }

  pub fn is_active(&self, module_graph: &ModuleGraph, runtime: &RuntimeSpec) -> bool {
    if !self.conditional {
      return self.active;
    }
    self
      .get_condition_state(module_graph, runtime)
      .is_not_false()
  }

  pub fn is_target_active(&self, module_graph: &ModuleGraph, runtime: &RuntimeSpec) -> bool {
    if !self.conditional {
      return self.active;
    }
    self.get_condition_state(module_graph, runtime).is_true()
  }

  pub fn get_active_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: &RuntimeSpec,
  ) -> ConnectionState {
    if !self.conditional {
      return ConnectionState::Bool(self.active);
    }
    self.get_condition_state(module_graph, runtime)
  }

  /// ## Panic
  /// This function will panic if we can't get condition from module graph
  /// Here avoid move condition, so use dependency id to search
  pub fn get_condition_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: &RuntimeSpec,
  ) -> ConnectionState {
    let condition_id = module_graph
      .connection_id_by_dependency_id(&self.dependency_id)
      .expect("should have corresponding connection id");
    let condition = module_graph
      .get_condition_by_connection_id(condition_id)
      .expect("should have condition");
    match condition {
      DependencyCondition::False => ConnectionState::Bool(false),
      DependencyCondition::Fn(f) => f(self, runtime, module_graph),
    }
  }
}

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
