use dyn_clone::clone_trait_object;

use super::Dependency;
use crate::DependencyType;

pub trait NullDependency: Dependency {
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Null
  }
}

clone_trait_object!(NullDependency);

pub trait AsNullDependency {
  fn as_null_dependency(&self) -> Option<&dyn NullDependency> {
    None
  }

  fn as_null_dependency_mut(&mut self) -> Option<&mut dyn NullDependency> {
    None
  }
}

impl<T: NullDependency> AsNullDependency for T {
  fn as_null_dependency(&self) -> Option<&dyn NullDependency> {
    Some(self)
  }

  fn as_null_dependency_mut(&mut self) -> Option<&mut dyn NullDependency> {
    Some(self)
  }
}
