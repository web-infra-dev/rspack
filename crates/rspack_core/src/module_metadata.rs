use std::sync::RwLock;

use rspack_cacheable::{cacheable, rkyv::with::Lock};

/// Module-owned metadata that can be updated through a shared module reference.
///
/// Mutation must still follow the compilation phase's rules. Synchronizing access
/// does not invalidate hashes or other results derived from the metadata.
#[cacheable]
#[derive(Debug, Default)]
pub struct ModuleMetadata<T> {
  #[cacheable(with=Lock)]
  value: RwLock<T>,
}

impl<T> From<T> for ModuleMetadata<T> {
  fn from(value: T) -> Self {
    Self {
      value: RwLock::new(value),
    }
  }
}

impl<T> ModuleMetadata<T> {
  /// Copies the current value without retaining a lock or sharing mutable state.
  pub fn snapshot(&self) -> T
  where
    T: Clone,
  {
    self
      .value
      .read()
      .expect("should read module metadata")
      .clone()
  }

  pub fn set(&self, value: T) {
    *self.value.write().expect("should write module metadata") = value;
  }
}
