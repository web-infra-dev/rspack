use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{DependencyId, ModuleIdentifier};

// FIXME: placing this as global id is not acceptable, move it to somewhere else later
static NEXT_MODULE_GRAPH_CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

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

#[derive(Debug, Clone, Eq)]
pub struct ModuleGraphConnection {
  /// The unique id of this connection
  pub id: ConnectionId,

  /// The referencing module identifier
  pub original_module_identifier: Option<ModuleIdentifier>,

  /// The referenced module identifier
  pub module_identifier: ModuleIdentifier,

  /// The referencing dependency id
  pub dependency_id: DependencyId,
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
    dependency_id: DependencyId,
    module_identifier: ModuleIdentifier,
  ) -> Self {
    let id = NEXT_MODULE_GRAPH_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
    Self {
      id: ConnectionId::from(id),
      original_module_identifier,
      module_identifier,
      dependency_id,
    }
  }
}
