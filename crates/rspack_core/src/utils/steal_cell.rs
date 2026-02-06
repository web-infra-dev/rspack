use std::{
  any::type_name,
  fmt::Debug,
  mem,
  ops::{Deref, DerefMut},
};

use crate::{
  ArtifactExt,
  incremental::{Incremental, IncrementalPasses},
};

/// A single-owner container that can be "stolen" exactly once.
///
/// `StealCell` keeps its inner value as `Some(T)` until `steal()` is called.
/// After stealing, any later read or write panics, which helps enforce phase
/// boundaries in compilation.
#[derive(Debug)]
pub struct StealCell<T>(Option<T>);

impl<T> From<T> for StealCell<T> {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

impl<T> StealCell<T> {
  pub fn new(value: T) -> Self {
    Self(Some(value))
  }

  pub fn is_stolen(&self) -> bool {
    self.0.is_none()
  }

  #[track_caller]
  pub fn steal(&mut self) -> T {
    self
      .0
      .take()
      .unwrap_or_else(|| panic!("attempt to steal from stolen value"))
  }
}

impl<T> Deref for StealCell<T> {
  type Target = T;

  #[track_caller]
  fn deref(&self) -> &Self::Target {
    self
      .0
      .as_ref()
      .unwrap_or_else(|| panic!("attempted to read from stolen value: {}", type_name::<T>()))
  }
}

impl<T> DerefMut for StealCell<T> {
  #[track_caller]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self
      .0
      .as_mut()
      .unwrap_or_else(|| panic!("attempt to read from stolen value"))
  }
}

impl<T: ArtifactExt> ArtifactExt for StealCell<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(new, old);
    }
  }
}
