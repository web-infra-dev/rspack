use std::{
  collections::HashSet,
  hash::{BuildHasher, Hash},
};

use rustc_hash::FxBuildHasher;

/// A struct to collect incremental info.
///
/// The `added`, `updated`, `removed` are disjoint.
pub struct IncrementalInfo<T, S = FxBuildHasher> {
  /// The added data but never removed.
  added: HashSet<T, S>,
  /// The added data that has been removed.
  updated: HashSet<T, S>,
  /// The removed data that never added again.
  removed: HashSet<T, S>,
}

impl<T, S> std::fmt::Debug for IncrementalInfo<T, S>
where
  T: std::fmt::Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("IncrementalInfo")
      .field("added", &self.added)
      .field("updated", &self.updated)
      .field("removed", &self.removed)
      .finish()
  }
}

impl<T, S> Default for IncrementalInfo<T, S>
where
  S: Default,
{
  fn default() -> Self {
    Self {
      added: HashSet::default(),
      updated: HashSet::default(),
      removed: HashSet::default(),
    }
  }
}

impl<T, S> IncrementalInfo<T, S>
where
  T: Hash + Eq + Clone,
  S: BuildHasher + Default,
{
  /// Mark a data as added.
  pub fn mark_as_add(&mut self, data: &T) {
    if self.removed.remove(data) {
      self.updated.insert(data.clone());
      return;
    }
    if self.updated.contains(data) {
      return;
    }
    self.added.insert(data.clone());
  }

  /// Mark a data as removed.
  pub fn mark_as_remove(&mut self, data: &T) {
    if self.added.remove(data) {
      return;
    }
    self.updated.remove(data);
    self.removed.insert(data.clone());
  }

  /// Reset incremental info.
  pub fn reset(&mut self) {
    self.added = HashSet::default();
    self.updated = HashSet::default();
    self.removed = HashSet::default();
  }

  /// Get added data.
  pub fn added(&self) -> &HashSet<T, S> {
    &self.added
  }

  /// Get updated data.
  pub fn updated(&self) -> &HashSet<T, S> {
    &self.updated
  }

  /// Get removed data.
  pub fn removed(&self) -> &HashSet<T, S> {
    &self.removed
  }

  /// Get active data (newly added + updated data).
  pub fn active(&self) -> impl Iterator<Item = &T> {
    self.added.iter().chain(self.updated.iter())
  }

  /// Get dirty data (removed + updated data).
  pub fn dirty(&self) -> impl Iterator<Item = &T> {
    self.removed.iter().chain(self.updated.iter())
  }
}

#[cfg(test)]
mod test {
  use super::IncrementalInfo;
  #[test]
  fn incremental_info_is_available() {
    let mut info = IncrementalInfo::<String>::default();
    let a = String::from("a");

    info.mark_as_add(&a);
    info.mark_as_add(&a);
    assert_eq!(info.added().len(), 1);
    assert_eq!(info.updated().len(), 0);
    assert_eq!(info.removed().len(), 0);

    info.mark_as_remove(&a);
    assert_eq!(info.added().len(), 0);
    assert_eq!(info.updated().len(), 0);
    assert_eq!(info.removed().len(), 0);

    info.mark_as_remove(&a);
    assert_eq!(info.added().len(), 0);
    assert_eq!(info.updated().len(), 0);
    assert_eq!(info.removed().len(), 1);

    info.mark_as_add(&a);
    assert_eq!(info.added().len(), 0);
    assert_eq!(info.updated().len(), 1);
    assert_eq!(info.removed().len(), 0);

    info.mark_as_remove(&a);
    assert_eq!(info.added().len(), 0);
    assert_eq!(info.updated().len(), 0);
    assert_eq!(info.removed().len(), 1);

    info.reset();
    assert_eq!(info.added().len(), 0);
    assert_eq!(info.updated().len(), 0);
    assert_eq!(info.removed().len(), 0);
  }
}
