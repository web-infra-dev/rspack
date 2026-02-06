use std::{
  any::type_name,
  fmt::Debug,
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
      .unwrap_or_else(|| panic!("attempt to steal from stolen value: {}", type_name::<T>()))
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
    self.0.as_mut().unwrap_or_else(|| {
      panic!(
        "attempted to mutably read from stolen value: {}",
        type_name::<T>()
      )
    })
  }
}

impl<T: ArtifactExt> ArtifactExt for StealCell<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn should_recover(incremental: &Incremental) -> bool {
    T::should_recover(incremental)
  }

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if new.is_stolen() || old.is_stolen() {
      panic!("attempted to recover stolen artifact: {}", type_name::<T>());
    }
    T::recover(incremental, &mut **new, &mut **old);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use super::StealCell;
  use crate::{
    ArtifactExt,
    incremental::{Incremental, IncrementalOptions, IncrementalPasses},
  };

  #[test]
  fn should_steal_once() {
    let mut cell = StealCell::new(1);
    assert_eq!(*cell, 1);
    assert_eq!(cell.steal(), 1);
    assert!(cell.is_stolen());
  }

  #[test]
  #[should_panic(expected = "attempt to steal from stolen value")]
  fn should_panic_when_steal_twice() {
    let mut cell = StealCell::new(1);
    let _ = cell.steal();
    let _ = cell.steal();
  }

  #[test]
  #[should_panic(expected = "attempted to read from stolen value")]
  fn should_panic_when_deref_stolen_cell() {
    let mut cell = StealCell::new(1);
    let _ = cell.steal();
    let _ = *cell;
  }

  #[derive(Debug)]
  struct TestArtifact {
    value: i32,
  }

  #[derive(Debug)]
  struct NeverRecoverArtifact;

  impl ArtifactExt for NeverRecoverArtifact {
    const PASS: IncrementalPasses = IncrementalPasses::BUILD_MODULE_GRAPH;

    fn should_recover(_incremental: &Incremental) -> bool {
      false
    }
  }

  #[derive(Debug)]
  struct AlwaysRecoverArtifact;

  impl ArtifactExt for AlwaysRecoverArtifact {
    const PASS: IncrementalPasses = IncrementalPasses::BUILD_MODULE_GRAPH;

    fn should_recover(_incremental: &Incremental) -> bool {
      true
    }
  }

  static RECOVER_CALLS: AtomicUsize = AtomicUsize::new(0);

  impl ArtifactExt for TestArtifact {
    const PASS: IncrementalPasses = IncrementalPasses::BUILD_MODULE_GRAPH;

    fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
      if Self::should_recover(incremental) {
        RECOVER_CALLS.fetch_add(1, Ordering::Relaxed);
        new.value += old.value;
      }
    }
  }

  #[test]
  fn should_delegate_recover_to_inner_artifact() {
    RECOVER_CALLS.store(0, Ordering::Relaxed);

    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::BUILD_MODULE_GRAPH,
    });
    let mut new = StealCell::new(TestArtifact { value: 1 });
    let mut old = StealCell::new(TestArtifact { value: 2 });

    <StealCell<TestArtifact> as ArtifactExt>::recover(&incremental, &mut new, &mut old);

    assert_eq!(RECOVER_CALLS.load(Ordering::Relaxed), 1);
    assert_eq!(new.value, 3);
    assert_eq!(old.value, 2);
  }

  #[test]
  #[should_panic(expected = "attempted to recover stolen artifact")]
  fn should_panic_when_recovering_stolen_artifact() {
    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::BUILD_MODULE_GRAPH,
    });
    let mut new = StealCell::new(TestArtifact { value: 1 });
    let mut old = StealCell::new(TestArtifact { value: 2 });
    let _ = old.steal();

    <StealCell<TestArtifact> as ArtifactExt>::recover(&incremental, &mut new, &mut old);
  }

  #[test]
  fn should_delegate_should_recover_to_inner_artifact() {
    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::all(),
    });

    assert!(!<StealCell<NeverRecoverArtifact> as ArtifactExt>::should_recover(&incremental));
  }

  #[test]
  fn should_use_inner_should_recover_instead_of_default_pass_logic() {
    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::empty(),
    });

    assert!(<StealCell<AlwaysRecoverArtifact> as ArtifactExt>::should_recover(&incremental));
  }
}
