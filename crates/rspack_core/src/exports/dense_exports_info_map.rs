use rayon::prelude::*;

use super::ExportsInfo;

#[derive(Debug, Clone)]
pub(crate) struct DenseExportsInfoMap<V> {
  offset: Option<u32>,
  values: Vec<Option<V>>,
}

impl<V> Default for DenseExportsInfoMap<V> {
  fn default() -> Self {
    Self {
      offset: None,
      values: Vec::new(),
    }
  }
}

impl<V> DenseExportsInfoMap<V> {
  #[inline]
  pub fn insert(&mut self, key: ExportsInfo, value: V) -> Option<V> {
    let index = self.ensure_index(key);
    self.values[index].replace(value)
  }

  #[inline]
  pub fn get(&self, key: &ExportsInfo) -> Option<&V> {
    self
      .index(*key)
      .and_then(|index| self.values.get(index))
      .and_then(Option::as_ref)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &ExportsInfo) -> Option<&mut V> {
    self
      .index(*key)
      .and_then(|index| self.values.get_mut(index))
      .and_then(Option::as_mut)
  }

  #[inline]
  pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = (ExportsInfo, &mut V)> + '_
  where
    V: Send,
  {
    let offset = self.offset.unwrap_or_default();
    self
      .values
      .par_iter_mut()
      .enumerate()
      .filter_map(move |(index, value)| {
        value
          .as_mut()
          .map(|value| (ExportsInfo::from_u32(offset + index as u32), value))
      })
  }

  #[inline]
  fn index(&self, key: ExportsInfo) -> Option<usize> {
    let offset = self.offset?;
    let key = key.as_u32();
    if key < offset {
      return None;
    }
    let index = (key - offset) as usize;
    (index < self.values.len()).then_some(index)
  }

  #[inline]
  fn ensure_index(&mut self, key: ExportsInfo) -> usize {
    let key = key.as_u32();
    let Some(offset) = self.offset else {
      self.offset = Some(key);
      self.values.resize_with(1, || None);
      return 0;
    };

    if key < offset {
      let additional = (offset - key) as usize;
      let old_values = std::mem::take(&mut self.values);
      self.values = Vec::with_capacity(additional + old_values.len());
      self.values.resize_with(additional, || None);
      self.values.extend(old_values);
      self.offset = Some(key);
      return 0;
    }

    let index = (key - offset) as usize;
    if self.values.len() <= index {
      self.values.resize_with(index + 1, || None);
    }
    index
  }
}

#[cfg(test)]
mod tests {
  use rayon::iter::ParallelIterator;

  use super::{DenseExportsInfoMap, ExportsInfo};

  #[test]
  fn supports_sparse_and_shifted_exports_info_ids() {
    let mut map = DenseExportsInfoMap::default();
    let a = ExportsInfo::from_u32(10);
    let b = ExportsInfo::from_u32(13);
    let c = ExportsInfo::from_u32(8);

    assert_eq!(map.insert(a, "a"), None);
    assert_eq!(map.insert(b, "b"), None);
    assert_eq!(map.insert(c, "c"), None);

    assert_eq!(map.get(&a), Some(&"a"));
    assert_eq!(map.get(&b), Some(&"b"));
    assert_eq!(map.get(&c), Some(&"c"));
    assert_eq!(map.get(&ExportsInfo::from_u32(9)), None);
  }

  #[test]
  fn par_iter_mut_visits_existing_values() {
    let mut map = DenseExportsInfoMap::default();
    map.insert(ExportsInfo::from_u32(20), 1);
    map.insert(ExportsInfo::from_u32(22), 3);

    map.par_iter_mut().for_each(|(_, value)| *value += 1);

    assert_eq!(map.get(&ExportsInfo::from_u32(20)), Some(&2));
    assert_eq!(map.get(&ExportsInfo::from_u32(22)), Some(&4));
  }
}
