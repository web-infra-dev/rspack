use std::sync::{
  RwLock,
  atomic::{AtomicBool, Ordering},
};

use rspack_cacheable::{
  cacheable,
  rkyv::with::{AtomicLoad, Lock, Relaxed},
};
use rspack_error::Diagnostic;

#[cacheable]
#[derive(Debug, Default)]
pub struct DependencyLazyState {
  #[cacheable(with=AtomicLoad<Relaxed>)]
  value: AtomicBool,
}

impl Clone for DependencyLazyState {
  fn clone(&self) -> Self {
    Self {
      value: AtomicBool::new(self.get()),
    }
  }
}

impl DependencyLazyState {
  pub fn get(&self) -> bool {
    self.value.load(Ordering::Relaxed)
  }

  pub fn set(&self) {
    self.value.store(true, Ordering::Relaxed);
  }

  pub fn unset(&self) -> bool {
    self.value.swap(false, Ordering::Relaxed)
  }
}

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
