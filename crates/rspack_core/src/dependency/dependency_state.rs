use std::sync::RwLock;

use rspack_cacheable::{cacheable, rkyv::with::Lock};
use rspack_error::Diagnostic;

#[cacheable]
#[derive(Debug, Default)]
pub struct DependencyCriticalState {
  #[cacheable(with=Lock)]
  value: RwLock<Option<Diagnostic>>,
}

impl Clone for DependencyCriticalState {
  fn clone(&self) -> Self {
    Self {
      value: RwLock::new(self.get()),
    }
  }
}

impl DependencyCriticalState {
  pub fn get(&self) -> Option<Diagnostic> {
    self.value.read().expect("should get read lock").clone()
  }

  pub fn set(&self, value: Option<Diagnostic>) {
    *self.value.write().expect("should get write lock") = value;
  }
}
