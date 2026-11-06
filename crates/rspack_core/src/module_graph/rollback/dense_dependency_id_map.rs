use super::{DenseStorageStats, PagedSlots};
use crate::DependencyId;

/// `DependencyId`-indexed map backed by [`PagedSlots`].
///
/// `DependencyId`s are allocated by a counter that never goes back, so storage
/// must release the slots of removed dependencies instead of keeping one slot
/// per id ever allocated.
#[derive(Debug)]
pub struct DenseDependencyIdMap<V> {
  values: PagedSlots<V>,
}

impl<V> Default for DenseDependencyIdMap<V> {
  fn default() -> Self {
    Self {
      values: PagedSlots::default(),
    }
  }
}

impl<V> DenseDependencyIdMap<V> {
  #[inline]
  pub fn insert(&mut self, key: DependencyId, value: V) -> Option<V> {
    self.values.insert(key.as_u32() as usize, value)
  }

  #[inline]
  pub fn remove(&mut self, key: &DependencyId) -> Option<V> {
    self.values.remove(key.as_u32() as usize)
  }

  #[inline]
  pub fn get(&self, key: &DependencyId) -> Option<&V> {
    self.values.get(key.as_u32() as usize)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &DependencyId) -> Option<&mut V> {
    self.values.get_mut(key.as_u32() as usize)
  }

  #[inline]
  pub fn clear(&mut self) {
    self.values.clear();
  }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (DependencyId, &V)> {
    self
      .values
      .iter()
      .map(|(index, value)| (DependencyId::from(index as u32), value))
  }

  pub fn stats(&self) -> DenseStorageStats {
    self.values.stats()
  }
}

#[cfg(test)]
mod tests {
  use crate::{DependencyId, module_graph::rollback::DenseDependencyIdMap};

  #[test]
  fn supports_sparse_dependency_ids() {
    let mut map = DenseDependencyIdMap::default();
    let a = DependencyId::from(1);
    let b = DependencyId::from(8);

    assert_eq!(map.insert(b, "b"), None);
    assert_eq!(map.insert(a, "a"), None);

    assert_eq!(map.get(&a), Some(&"a"));
    assert_eq!(map.get(&b), Some(&"b"));
    assert_eq!(map.get(&DependencyId::from(3)), None);

    assert_eq!(map.iter().collect::<Vec<_>>(), vec![(a, &"a"), (b, &"b")]);
  }

  #[test]
  fn remove_returns_existing_value() {
    let mut map = DenseDependencyIdMap::default();
    let id = DependencyId::from(2);

    map.insert(id, 1);
    assert_eq!(map.remove(&id), Some(1));
    assert_eq!(map.get(&id), None);
    assert_eq!(map.remove(&id), None);
  }
}
