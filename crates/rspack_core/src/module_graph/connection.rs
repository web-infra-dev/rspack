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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ModuleGraphConnection {
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,

  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,

  /// The referencing dependency id
  pub dependency_id: DependencyId,

  active: Option<bool>,
}

impl ModuleGraphConnection {
  pub fn new(
    original_module_identifier: Option<ModuleIdentifier>,
    dependency_id: DependencyId,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    Self {
      original_module_identifier,
      module_identifier,
      dependency_id,
      active: None,
    }
  }

  pub fn set_active(&mut self, value: bool) {
    self.active = Some(value);
  }

  pub fn is_active(&self, module_graph: &ModuleGraph, runtime: &RuntimeSpec) -> bool {
    if let Some(value) = self.active {
      return value;
    }
    self
      .get_condition_state(module_graph, runtime)
      .is_not_false()
  }

  pub fn is_target_active(&self, module_graph: &ModuleGraph, runtime: &RuntimeSpec) -> bool {
    if let Some(value) = self.active {
      return value;
    }
    self.get_condition_state(module_graph, runtime).is_true()
  }

  pub fn get_active_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: &RuntimeSpec,
  ) -> ConnectionState {
    if let Some(value) = self.active {
      return ConnectionState::Bool(value);
    }
    self.get_condition_state(module_graph, runtime)
  }

  // Here avoid move condition, so use dependency id to search
  pub fn get_condition_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: &RuntimeSpec,
  ) -> ConnectionState {
    let dependency = module_graph
      .dependency_by_id(&self.dependency_id)
      .expect("should have dependency");
    match dependency.get_condition(module_graph) {
      DependencyCondition::Nil => ConnectionState::Bool(false),
      DependencyCondition::False => ConnectionState::Bool(true),
      DependencyCondition::Fn(f) => f(self, runtime, module_graph),
    }
  }
}

pub enum ConnectionState {
  Bool(bool),
}

impl ConnectionState {
  pub fn is_true(&self) -> bool {
    match self {
      ConnectionState::Bool(bool) => *bool,
    }
    // other should return false
  }

  pub fn is_not_false(&self) -> bool {
    match self {
      ConnectionState::Bool(bool) => *bool,
    }
    // other should return false
  }
}
