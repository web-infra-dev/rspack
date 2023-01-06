use std::{any::Any, hash::Hash};

pub trait AsAny {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
  fn as_any(&self) -> &dyn Any {
    self
  }
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

pub trait DynHash {
  fn dyn_hash(&self, state: &mut dyn std::hash::Hasher);
}

impl<T: Hash> DynHash for T {
  fn dyn_hash(&self, mut state: &mut dyn std::hash::Hasher) {
    self.hash(&mut state);
  }
}

pub trait DynEq {
  fn dyn_eq(&self, other: &dyn Any) -> bool;
}

impl<T: Eq + Any> DynEq for T {
  fn dyn_eq(&self, other: &dyn Any) -> bool {
    if let Some(module) = other.downcast_ref::<T>() {
      self == module
    } else {
      false
    }
  }
}
