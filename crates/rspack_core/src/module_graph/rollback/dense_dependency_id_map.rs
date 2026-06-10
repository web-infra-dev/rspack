use crate::DependencyId;

#[derive(Debug, Clone)]
pub struct DenseDependencyIdMap<V> {
  values: Vec<Option<V>>,
}

impl<V> Default for DenseDependencyIdMap<V> {
  fn default() -> Self {
    Self { values: Vec::new() }
  }
}

impl<V> DenseDependencyIdMap<V> {
  #[inline]
  pub fn insert(&mut self, key: DependencyId, value: V) -> Option<V> {
    let index = key.as_u32() as usize;
    if self.values.len() <= index {
      self.values.resize_with(index + 1, || None);
    }
    self.values[index].replace(value)
  }

  #[inline]
  pub fn remove(&mut self, key: &DependencyId) -> Option<V> {
    self
      .values
      .get_mut(key.as_u32() as usize)
      .and_then(Option::take)
  }

  #[inline]
  pub fn get(&self, key: &DependencyId) -> Option<&V> {
    self
      .values
      .get(key.as_u32() as usize)
      .and_then(Option::as_ref)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &DependencyId) -> Option<&mut V> {
    self
      .values
      .get_mut(key.as_u32() as usize)
      .and_then(Option::as_mut)
  }

  #[inline]
  pub fn clear(&mut self) {
    self.values.clear();
  }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (DependencyId, &V)> {
    self.values.iter().enumerate().filter_map(|(index, value)| {
      value
        .as_ref()
        .map(|value| (DependencyId::from(index as u32), value))
    })
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
