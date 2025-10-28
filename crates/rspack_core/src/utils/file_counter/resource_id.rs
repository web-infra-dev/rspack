use crate::{DependencyId, ModuleIdentifier};

/// The resource using the path at file counter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceId {
  Module(ModuleIdentifier),
  Dependency(DependencyId),
}

impl From<ModuleIdentifier> for ResourceId {
  fn from(value: ModuleIdentifier) -> Self {
    Self::Module(value)
  }
}

impl From<&ModuleIdentifier> for ResourceId {
  fn from(value: &ModuleIdentifier) -> Self {
    Self::Module(*value)
  }
}

impl From<DependencyId> for ResourceId {
  fn from(value: DependencyId) -> Self {
    Self::Dependency(value)
  }
}

impl From<&DependencyId> for ResourceId {
  fn from(value: &DependencyId) -> Self {
    Self::Dependency(*value)
  }
}
