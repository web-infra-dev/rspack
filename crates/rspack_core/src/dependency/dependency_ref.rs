use std::{
  fmt,
  hash::{Hash, Hasher},
  sync::Arc,
};

use rspack_cacheable::cacheable;

use super::dependency_trait::{BoxDependency, Dependency};

/// A shared dependency whose identity is the allocation behind the `Arc`.
///
/// Cloning a `DependencyRef` preserves identity. Serializing and deserializing may
/// change the allocation address, but cacheable's shared-pointer table preserves
/// aliasing between references in the deserialized object graph.
#[cacheable]
#[derive(Clone)]
pub struct DependencyRef(Arc<dyn Dependency>);

impl DependencyRef {
  pub fn new(dependency: impl Dependency + 'static) -> Self {
    Self(Arc::new(dependency))
  }

  pub fn id(&self) -> &Self {
    self
  }

  pub fn as_dependency(&self) -> &(dyn Dependency + 'static) {
    self.0.as_ref()
  }

  pub fn ptr_eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0)
  }

  pub fn into_arc(self) -> Arc<dyn Dependency> {
    self.0
  }
}

impl From<BoxDependency> for DependencyRef {
  fn from(dependency: BoxDependency) -> Self {
    Self(dependency.into())
  }
}

impl From<Arc<dyn Dependency>> for DependencyRef {
  fn from(dependency: Arc<dyn Dependency>) -> Self {
    Self(dependency)
  }
}

impl AsRef<dyn Dependency> for DependencyRef {
  fn as_ref(&self) -> &(dyn Dependency + 'static) {
    self.as_dependency()
  }
}

impl PartialEq for DependencyRef {
  fn eq(&self, other: &Self) -> bool {
    self.ptr_eq(other)
  }
}

impl Eq for DependencyRef {}

impl Hash for DependencyRef {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (Arc::as_ptr(&self.0) as *const ()).hash(state);
  }
}

impl fmt::Debug for DependencyRef {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DependencyRef")
      .field("address", &format_args!("{:p}", Arc::as_ptr(&self.0)))
      .field("dependency", &self.0)
      .finish()
  }
}
