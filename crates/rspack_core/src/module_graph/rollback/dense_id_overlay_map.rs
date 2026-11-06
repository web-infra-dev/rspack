use std::{marker::PhantomData, ops::Deref};

use super::{DenseStorageStats, OverlayValue, PagedSlots};

/// Dense id-indexed map with a checkpoint overlay, backed by [`PagedSlots`].
///
/// Writes go to `base` until [`DenseIdOverlayMap::checkpoint`]. After that
/// they go to `overlay`, which [`DenseIdOverlayMap::reset`] discards. Neither
/// layer grows with the number of ids ever allocated.
///
/// The overlay is only allocated on the first write after a checkpoint. A map
/// that is never written during seal therefore pays only one `Option` check per
/// lookup for the overlay.
#[derive(Debug)]
pub struct DenseIdOverlayMap<K, V> {
  base: PagedSlots<V>,
  overlay: Option<PagedSlots<OverlayValue<V>>>,
  /// Whether writes go to the overlay.
  checkpointed: bool,
  key: PhantomData<K>,
}

impl<K, V> Default for DenseIdOverlayMap<K, V> {
  fn default() -> Self {
    Self {
      base: PagedSlots::default(),
      overlay: None,
      checkpointed: false,
      key: PhantomData,
    }
  }
}

impl<K: Deref<Target = u32>, V> DenseIdOverlayMap<K, V> {
  /// The first index beyond all allocated slots in the current graph snapshot.
  ///
  /// Removing entries does not lower it, so ids taken from it are never reused
  /// while the map is alive.
  #[inline]
  pub fn next_index(&self) -> usize {
    self
      .base
      .end()
      .max(self.overlay.as_ref().map_or(0, PagedSlots::end))
  }

  #[inline]
  pub fn checkpoint(&mut self) {
    self.checkpointed = true;
  }

  #[inline]
  pub fn reset(&mut self) {
    self.overlay = None;
    self.checkpointed = false;
  }

  #[inline]
  pub fn insert(&mut self, key: K, value: V) {
    let index = *key as usize;
    if self.checkpointed {
      self.overlay().insert(index, OverlayValue::Value(value));
    } else {
      self.base.insert(index, value);
    }
  }

  #[inline]
  pub fn remove(&mut self, key: &K) {
    let index = **key as usize;
    if self.checkpointed {
      self.overlay().insert(index, OverlayValue::Tombstone);
    } else {
      self.base.remove(index);
    }
  }

  #[inline]
  pub fn get(&self, key: &K) -> Option<&V> {
    let index = **key as usize;
    if let Some(overlay) = &self.overlay
      && let Some(value) = overlay.get(index)
    {
      return match value {
        OverlayValue::Value(value) => Some(value),
        OverlayValue::Tombstone => None,
      };
    }
    self.base.get(index)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
  where
    V: Clone,
  {
    let index = **key as usize;
    if !self.checkpointed {
      return self.base.get_mut(index);
    }
    if !self
      .overlay
      .as_ref()
      .is_some_and(|overlay| overlay.contains(index))
    {
      let value = self.base.get(index)?.clone();
      self.overlay().insert(index, OverlayValue::Value(value));
    }
    match self.overlay.as_mut()?.get_mut(index) {
      Some(OverlayValue::Value(value)) => Some(value),
      _ => None,
    }
  }

  /// Stats for the base storage and the checkpoint overlay, in that order.
  pub fn stats(&self) -> (DenseStorageStats, DenseStorageStats) {
    (
      self.base.stats(),
      self
        .overlay
        .as_ref()
        .map(PagedSlots::stats)
        .unwrap_or_default(),
    )
  }

  #[inline]
  fn overlay(&mut self) -> &mut PagedSlots<OverlayValue<V>> {
    self.overlay.get_or_insert_with(PagedSlots::default)
  }
}

#[cfg(test)]
mod tests {
  use crate::{DependencyId, module_graph::rollback::DenseIdOverlayMap};

  #[test]
  fn checkpoint_inserts_apply_only_to_overlay() {
    let mut map = DenseIdOverlayMap::default();
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
    let mut map = DenseIdOverlayMap::default();
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
    let mut map = DenseIdOverlayMap::default();
    let a = DependencyId::from(3);

    map.insert(a, 1);
    map.checkpoint();
    *map.get_mut(&a).expect("should clone base into overlay") = 5;

    assert_eq!(map.get(&a), Some(&5));

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
  }
}
