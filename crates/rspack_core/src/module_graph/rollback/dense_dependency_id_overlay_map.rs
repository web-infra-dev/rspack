use super::OverlayValue;
use crate::DependencyId;

#[derive(Debug, Clone)]
pub struct DenseDependencyIdOverlayMap<V> {
  base: Vec<Option<V>>,
  overlay: Option<Vec<Option<OverlayValue<V>>>>,
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
    let index = key.as_u32() as usize;
    if self.overlay.is_some() {
      Self::ensure_len(self.overlay(), index);
      self.overlay.as_mut().expect("overlay checked above")[index] =
        Some(OverlayValue::Value(value));
    } else {
      Self::ensure_len(&mut self.base, index);
      self.base[index] = Some(value);
    }
  }

  #[inline]
  pub fn remove(&mut self, key: &DependencyId) {
    let index = key.as_u32() as usize;
    if self.overlay.is_some() {
      Self::ensure_len(self.overlay(), index);
      self.overlay.as_mut().expect("overlay checked above")[index] = Some(OverlayValue::Tombstone);
    } else if let Some(value) = self.base.get_mut(index) {
      *value = None;
    }
  }

  #[inline]
  pub fn get(&self, key: &DependencyId) -> Option<&V> {
    let index = key.as_u32() as usize;
    if let Some(overlay) = &self.overlay
      && let Some(Some(value)) = overlay.get(index)
    {
      return match value {
        OverlayValue::Value(value) => Some(value),
        OverlayValue::Tombstone => None,
      };
    }
    self.base.get(index).and_then(Option::as_ref)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &DependencyId) -> Option<&mut V>
  where
    V: Clone,
  {
    let index = key.as_u32() as usize;
    if self.overlay.is_some() {
      self.materialize_overlay_value(index);
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      match overlay.get_mut(index).and_then(Option::as_mut) {
        Some(OverlayValue::Value(value)) => Some(value),
        _ => None,
      }
    } else {
      self.base.get_mut(index).and_then(Option::as_mut)
    }
  }

  #[inline]
  fn materialize_overlay_value(&mut self, index: usize)
  where
    V: Clone,
  {
    let overlay = self.overlay.as_ref().expect("overlay checked above");
    if matches!(overlay.get(index), Some(Some(_))) {
      return;
    }

    if let Some(value) = self.base.get(index).and_then(Option::as_ref).cloned() {
      Self::ensure_len(self.overlay(), index);
      self.overlay.as_mut().expect("overlay checked above")[index] =
        Some(OverlayValue::Value(value));
    }
  }

  #[inline]
  fn overlay(&mut self) -> &mut Vec<Option<OverlayValue<V>>> {
    self.overlay.get_or_insert_with(Vec::new)
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
}
