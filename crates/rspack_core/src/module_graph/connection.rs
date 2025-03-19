use std::hash::Hash;

use rspack_cacheable::cacheable;

use crate::{DependencyId, ModuleGraph, ModuleIdentifier, RuntimeSpec};

#[cacheable]
#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  pub dependency_id: DependencyId,
  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub resolved_original_module_identifier: Option<ModuleIdentifier>,
  pub resolved_module: ModuleIdentifier,

  /// The referenced module identifier
  module_identifier: ModuleIdentifier,

  pub active: bool,
  pub conditional: bool,
}

impl Hash for ModuleGraphConnection {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.dependency_id.hash(state);
  }
}

impl PartialEq for ModuleGraphConnection {
  fn eq(&self, other: &Self) -> bool {
    self.dependency_id == other.dependency_id
  }
}

impl ModuleGraphConnection {
  pub fn new(
    dependency_id: DependencyId,
    original_module_identifier: Option<ModuleIdentifier>,
    module_identifier: ModuleIdentifier,
    active: bool,
    conditional: bool,
  ) -> Self {
    Self {
      dependency_id,
      original_module_identifier,
      module_identifier,
      active,
      conditional,
      resolved_original_module_identifier: original_module_identifier,
      resolved_module: module_identifier,
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
    module_graph
      .get_condition_state(self, runtime)
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
    module_graph.get_condition_state(self, runtime).is_true()
  }

  pub fn active_state(
    &self,
    module_graph: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> ConnectionState {
    if !self.conditional {
      return ConnectionState::Bool(self.active);
    }

    module_graph.get_condition_state(self, runtime)
  }

  pub fn module_identifier(&self) -> &ModuleIdentifier {
    &self.module_identifier
  }

  /// used for set module identifier after clone the [ModuleGraphConnection]
  pub fn set_module_identifier(&mut self, mi: ModuleIdentifier) {
    self.module_identifier = mi;
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl std::ops::Add for ConnectionState {
  type Output = Self;
  fn add(self, other: Self) -> Self::Output {
    if matches!(self, ConnectionState::Bool(true)) || matches!(other, ConnectionState::Bool(true)) {
      return ConnectionState::Bool(true);
    }
    if matches!(self, ConnectionState::Bool(false)) {
      return other;
    }
    if matches!(other, ConnectionState::Bool(false)) {
      return self;
    }
    if matches!(self, ConnectionState::TransitiveOnly) {
      return other;
    }
    if matches!(other, ConnectionState::TransitiveOnly) {
      return self;
    }
    self
  }
}
