use crate::DependencyId;

const UNCHANGED: u8 = 0;
const CHANGED: u8 = 1;

#[derive(Debug, Clone)]
struct UndoEntry<V> {
  index: u32,
  value: Option<V>,
}

#[derive(Debug, Clone)]
struct DenseRollback<V> {
  original_len: usize,
  /// Marks dependency ids whose original values are already in `entries`.
  changed: Vec<u8>,
  /// Stores only original values that must be restored.
  entries: Vec<UndoEntry<V>>,
}

impl<V> DenseRollback<V> {
  fn new(original_len: usize) -> Self {
    Self {
      original_len,
      changed: Vec::new(),
      entries: Vec::new(),
    }
  }

  #[inline]
  fn has_changed(&self, index: usize) -> bool {
    self.changed.get(index).copied().unwrap_or(UNCHANGED) == CHANGED
  }

  #[inline]
  fn record(&mut self, index: u32, value: Option<V>) {
    let dense_index = index as usize;
    if self.changed.len() <= dense_index {
      self.changed.resize(dense_index + 1, UNCHANGED);
    }

    debug_assert_eq!(self.changed[dense_index], UNCHANGED);
    self.changed[dense_index] = CHANGED;
    self.entries.push(UndoEntry { index, value });
  }
}

/// A dense dependency map whose checkpointed mutations are applied in place.
/// Original values are recorded once so reads keep the same direct lookup path
/// before and after a checkpoint.
#[derive(Debug, Clone)]
pub struct DenseDependencyIdOverlayMap<V> {
  values: Vec<Option<V>>,
  rollback: Option<DenseRollback<V>>,
}

impl<V> Default for DenseDependencyIdOverlayMap<V> {
  fn default() -> Self {
    Self {
      values: Vec::new(),
      rollback: None,
    }
  }
}

impl<V> DenseDependencyIdOverlayMap<V> {
  #[inline]
  pub fn checkpoint(&mut self) {
    if self.rollback.is_none() {
      self.rollback = Some(DenseRollback::new(self.values.len()));
    }
  }

  #[inline]
  pub fn reset(&mut self) {
    let Some(rollback) = self.rollback.take() else {
      return;
    };
    let DenseRollback {
      original_len,
      entries,
      ..
    } = rollback;

    for entry in entries {
      self.values[entry.index as usize] = entry.value;
    }
    self.values.truncate(original_len);
  }

  #[inline]
  pub fn insert(&mut self, key: DependencyId, value: V) {
    let index = key.as_u32() as usize;
    Self::ensure_len(&mut self.values, index);
    let previous = self.values[index].replace(value);

    if let Some(rollback) = &mut self.rollback
      && !rollback.has_changed(index)
    {
      rollback.record(key.as_u32(), previous);
    }
  }

  #[inline]
  pub fn remove(&mut self, key: &DependencyId) {
    let index = key.as_u32() as usize;
    let Some(value) = self.values.get_mut(index) else {
      return;
    };
    let Some(previous) = value.take() else {
      return;
    };

    if let Some(rollback) = &mut self.rollback
      && !rollback.has_changed(index)
    {
      rollback.record(key.as_u32(), Some(previous));
    }
  }

  #[inline]
  pub fn get(&self, key: &DependencyId) -> Option<&V> {
    let index = key.as_u32() as usize;
    self.values.get(index).and_then(Option::as_ref)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &DependencyId) -> Option<&mut V>
  where
    V: Clone,
  {
    let index = key.as_u32() as usize;
    if self
      .rollback
      .as_ref()
      .is_some_and(|rollback| !rollback.has_changed(index))
    {
      let original = self.values.get(index)?.as_ref()?.clone();
      self
        .rollback
        .as_mut()
        .expect("rollback checked above")
        .record(key.as_u32(), Some(original));
    }
    self.values.get_mut(index).and_then(Option::as_mut)
  }

  #[inline]
  fn ensure_len<T>(values: &mut Vec<Option<T>>, index: usize) {
    if values.len() <= index {
      values.resize_with(index + 1, || None);
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{DependencyId, module_graph::rollback::DenseDependencyIdOverlayMap};

  #[test]
  fn checkpoint_inserts_are_rolled_back() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let a = DependencyId::from(0);
    let b = DependencyId::from(1);

    map.insert(a, 1);
    map.checkpoint();
    map.insert(b, 2);
    map.insert(a, 3);

    assert_eq!(map.get(&a), Some(&3));
    assert_eq!(map.get(&b), Some(&2));

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
    assert_eq!(map.get(&b), None);
  }

  #[test]
  fn remove_after_checkpoint_is_rolled_back() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let a = DependencyId::from(0);
    let b = DependencyId::from(7);

    map.insert(a, 1);
    map.insert(b, 2);
    map.checkpoint();
    map.remove(&a);

    assert_eq!(map.get(&a), None);
    assert_eq!(map.get(&b), Some(&2));

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
    assert_eq!(map.get(&b), Some(&2));
  }

  #[test]
  fn get_mut_records_the_original_value() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let a = DependencyId::from(3);

    map.insert(a, 1);
    map.checkpoint();
    *map.get_mut(&a).expect("should record the original value") = 5;

    assert_eq!(map.get(&a), Some(&5));

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
  }

  #[test]
  fn repeated_get_mut_records_the_original_value_once() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let id = DependencyId::from(3);

    map.insert(id, 1);
    map.checkpoint();
    *map.get_mut(&id).expect("should record the original value") = 2;
    *map.get_mut(&id).expect("should reuse the rollback entry") = 3;

    let rollback = map.rollback.as_ref().expect("should have rollback data");
    assert_eq!(rollback.entries.len(), 1);

    map.reset();

    assert_eq!(map.get(&id), Some(&1));
  }

  #[test]
  fn repeated_writes_record_the_original_value_once() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let id = DependencyId::from(7);

    map.checkpoint();
    map.insert(id, 1);
    map.insert(id, 2);
    map.remove(&id);

    let rollback = map.rollback.as_ref().expect("should have rollback data");
    assert_eq!(rollback.changed.len(), 8);
    assert_eq!(rollback.entries.len(), 1);
    assert_eq!(map.get(&id), None);
  }

  #[test]
  fn sparse_rollback_stores_only_changed_values() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let id = DependencyId::from(1024);

    map.checkpoint();
    map.insert(id, 1);

    let rollback = map.rollback.as_ref().expect("should have rollback data");
    assert_eq!(rollback.changed.len(), 1025);
    assert_eq!(rollback.entries.len(), 1);
    assert_eq!(map.get(&id), Some(&1));

    map.reset();

    assert!(map.values.is_empty());
    assert_eq!(map.get(&id), None);
  }
}
