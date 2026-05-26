use super::OverlayValue;
use crate::DependencyId;

const PAGE_BITS: usize = 10;
const PAGE_SIZE: usize = 1 << PAGE_BITS;
const PAGE_MASK: usize = PAGE_SIZE - 1;

#[derive(Debug, Clone)]
struct DenseDependencyIdPage<T> {
  values: Vec<Option<T>>,
  len: usize,
}

impl<T> Default for DenseDependencyIdPage<T> {
  fn default() -> Self {
    Self {
      values: (0..PAGE_SIZE).map(|_| None).collect(),
      len: 0,
    }
  }
}

impl<T> DenseDependencyIdPage<T> {
  #[inline]
  fn insert(&mut self, index: usize, value: T) {
    if self.values[index].is_none() {
      self.len += 1;
    }
    self.values[index] = Some(value);
  }

  #[inline]
  fn remove(&mut self, index: usize) {
    if self.values[index].take().is_some() {
      self.len -= 1;
    }
  }

  #[inline]
  fn get(&self, index: usize) -> Option<&T> {
    self.values[index].as_ref()
  }

  #[inline]
  fn get_mut(&mut self, index: usize) -> Option<&mut T> {
    self.values[index].as_mut()
  }

  #[inline]
  fn is_empty(&self) -> bool {
    self.len == 0
  }
}

#[derive(Debug, Clone)]
pub struct DenseDependencyIdOverlayMap<V> {
  base: Vec<Option<DenseDependencyIdPage<V>>>,
  overlay: Option<Vec<Option<DenseDependencyIdPage<OverlayValue<V>>>>>,
}

impl<V> Default for DenseDependencyIdOverlayMap<V> {
  fn default() -> Self {
    Self {
      base: Vec::new(),
      overlay: None,
    }
  }
}

impl<V> DenseDependencyIdOverlayMap<V> {
  #[inline]
  pub fn checkpoint(&mut self) {
    self.overlay.get_or_insert_with(Vec::new);
  }

  #[inline]
  pub fn reset(&mut self) {
    self.overlay = None;
  }

  #[inline]
  pub fn insert(&mut self, key: DependencyId, value: V) {
    let (page_index, value_index) = Self::indexes(key);
    if self.overlay.is_some() {
      Self::ensure_page(self.overlay(), page_index).insert(value_index, OverlayValue::Value(value));
    } else {
      Self::ensure_page(&mut self.base, page_index).insert(value_index, value);
    }
  }

  #[inline]
  pub fn remove(&mut self, key: &DependencyId) {
    let (page_index, value_index) = Self::indexes(*key);
    if self.overlay.is_some() {
      Self::ensure_page(self.overlay(), page_index).insert(value_index, OverlayValue::Tombstone);
    } else if let Some(page) = self.base.get_mut(page_index).and_then(Option::as_mut) {
      page.remove(value_index);
      if page.is_empty() {
        self.base[page_index] = None;
        Self::trim_trailing_empty_pages(&mut self.base);
      }
    }
  }

  #[inline]
  pub fn get(&self, key: &DependencyId) -> Option<&V> {
    let (page_index, value_index) = Self::indexes(*key);
    if let Some(overlay) = &self.overlay
      && let Some(value) = Self::get_page_value(overlay, page_index, value_index)
    {
      return match value {
        OverlayValue::Value(value) => Some(value),
        OverlayValue::Tombstone => None,
      };
    }
    Self::get_page_value(&self.base, page_index, value_index)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &DependencyId) -> Option<&mut V>
  where
    V: Clone,
  {
    let (page_index, value_index) = Self::indexes(*key);
    if self.overlay.is_some() {
      self.materialize_overlay_value(page_index, value_index);
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      match Self::get_page_value_mut(overlay, page_index, value_index) {
        Some(OverlayValue::Value(value)) => Some(value),
        _ => None,
      }
    } else {
      Self::get_page_value_mut(&mut self.base, page_index, value_index)
    }
  }

  #[inline]
  fn materialize_overlay_value(&mut self, page_index: usize, value_index: usize)
  where
    V: Clone,
  {
    let overlay = self.overlay.as_ref().expect("overlay checked above");
    if Self::get_page_value(overlay, page_index, value_index).is_some() {
      return;
    }

    if let Some(value) = Self::get_page_value(&self.base, page_index, value_index).cloned() {
      Self::ensure_page(self.overlay(), page_index).insert(value_index, OverlayValue::Value(value));
    }
  }

  #[inline]
  fn overlay(&mut self) -> &mut Vec<Option<DenseDependencyIdPage<OverlayValue<V>>>> {
    self.overlay.get_or_insert_with(Vec::new)
  }

  #[inline]
  fn indexes(key: DependencyId) -> (usize, usize) {
    let index = key.as_u32() as usize;
    (index >> PAGE_BITS, index & PAGE_MASK)
  }

  #[inline]
  fn ensure_page<T>(
    pages: &mut Vec<Option<DenseDependencyIdPage<T>>>,
    page_index: usize,
  ) -> &mut DenseDependencyIdPage<T> {
    if pages.len() <= page_index {
      pages.resize_with(page_index + 1, || None);
    }
    pages[page_index].get_or_insert_with(DenseDependencyIdPage::default)
  }

  #[inline]
  fn get_page_value<T>(
    pages: &[Option<DenseDependencyIdPage<T>>],
    page_index: usize,
    value_index: usize,
  ) -> Option<&T> {
    pages
      .get(page_index)
      .and_then(Option::as_ref)
      .and_then(|page| page.get(value_index))
  }

  #[inline]
  fn get_page_value_mut<T>(
    pages: &mut [Option<DenseDependencyIdPage<T>>],
    page_index: usize,
    value_index: usize,
  ) -> Option<&mut T> {
    pages
      .get_mut(page_index)
      .and_then(Option::as_mut)
      .and_then(|page| page.get_mut(value_index))
  }

  #[inline]
  fn trim_trailing_empty_pages<T>(pages: &mut Vec<Option<DenseDependencyIdPage<T>>>) {
    while pages.last().is_some_and(Option::is_none) {
      pages.pop();
    }
  }

  #[cfg(test)]
  fn base_pages_len(&self) -> usize {
    self.base.len()
  }
}

#[cfg(test)]
mod tests {
  use crate::{DependencyId, module_graph::rollback::DenseDependencyIdOverlayMap};

  #[test]
  fn checkpoint_inserts_apply_only_to_overlay() {
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
  fn remove_in_overlay_masks_base() {
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
  fn get_mut_clones_base_into_overlay() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let a = DependencyId::from(3);

    map.insert(a, 1);
    map.checkpoint();
    *map.get_mut(&a).expect("should clone base into overlay") = 5;

    assert_eq!(map.get(&a), Some(&5));

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
  }

  #[test]
  fn remove_releases_trailing_empty_pages() {
    let mut map = DenseDependencyIdOverlayMap::default();
    let low = DependencyId::from(0);
    let high = DependencyId::from((super::PAGE_SIZE * 3) as u32);

    map.insert(low, 1);
    map.insert(high, 2);
    assert_eq!(map.base_pages_len(), 4);

    map.remove(&high);
    assert_eq!(map.base_pages_len(), 1);
    assert_eq!(map.get(&low), Some(&1));
    assert_eq!(map.get(&high), None);
  }
}
