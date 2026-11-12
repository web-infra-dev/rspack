use std::{
  collections::{HashMap, hash_map::Iter as HashMapIter},
  fmt::Debug,
  hash::{BuildHasher, Hash},
};

use rustc_hash::FxBuildHasher;

#[derive(Debug)]
pub enum OverlayValue<V> {
  Value(V),
  Tombstone,
}

#[derive(Debug)]
pub struct OverlayMap<K, V, S = FxBuildHasher> {
  base: HashMap<K, V, S>,
  overlay: Option<HashMap<K, OverlayValue<V>, S>>,
}

impl<K, V, S> Default for OverlayMap<K, V, S>
where
  K: Eq + Hash,
  S: BuildHasher + Default,
{
  fn default() -> Self {
    Self {
      base: HashMap::default(),
      overlay: None,
    }
  }
}

#[derive(Debug)]
pub enum OverlayIter<'a, K, V, S = FxBuildHasher> {
  Base(HashMapIter<'a, K, V>),
  Combined {
    overlay_keys: &'a HashMap<K, OverlayValue<V>, S>,
    overlay_iter: HashMapIter<'a, K, OverlayValue<V>>,
    base_iter: HashMapIter<'a, K, V>,
  },
}

impl<'a, K, V, S> Iterator for OverlayIter<'a, K, V, S>
where
  K: Eq + Hash,
  S: BuildHasher,
{
  type Item = (&'a K, &'a V);

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::Base(iter) => iter.next(),
      Self::Combined {
        overlay_keys,
        overlay_iter,
        base_iter,
      } => loop {
        if let Some((key, value)) = overlay_iter.next() {
          if let OverlayValue::Value(value) = value {
            return Some((key, value));
          }
          continue;
        }

        for (key, value) in base_iter {
          if overlay_keys.contains_key(key) {
            continue;
          }
          return Some((key, value));
        }

        return None;
      },
    }
  }
}

impl<K, V, S> OverlayMap<K, V, S>
where
  K: Eq + Hash + Clone,
  S: BuildHasher + Default + Clone,
{
  pub fn new(base: HashMap<K, V, S>) -> Self {
    Self {
      base,
      overlay: None,
    }
  }

  /// Enable overlay mode so subsequent mutations stay in the overlay.
  pub fn checkpoint(&mut self) {
    self.overlay.get_or_insert_with(HashMap::default);
  }

  /// Drop the overlay without applying its changes.
  pub fn reset(&mut self) {
    self.overlay = None;
  }

  /// Fold the overlay into the base map and continue recording into a fresh
  /// overlay, so a later `reset` only discards changes made after this point.
  ///
  /// A no-op when no checkpoint is active: enabling recording here would
  /// diverge from containers whose rebase is a no-op without a checkpoint,
  /// and split subsequent writes between the overlay and the base map.
  ///
  /// Used when a sanctioned post-checkpoint mutation (e.g. `rebuild_module`)
  /// must become part of the rollback baseline instead of being undone.
  pub fn rebase(&mut self) {
    if let Some(overlay) = self.overlay.take() {
      for (key, value) in overlay {
        match value {
          OverlayValue::Value(value) => {
            self.base.insert(key, value);
          }
          OverlayValue::Tombstone => {
            self.base.remove(&key);
          }
        }
      }
      self.overlay.get_or_insert_with(HashMap::default);
    }
  }

  #[allow(dead_code)]
  pub fn contains_key(&self, key: &K) -> bool {
    self.get(key).is_some()
  }

  pub fn insert(&mut self, key: K, value: V) {
    if self.overlay.is_some() {
      let overlay = self.overlay();
      overlay.insert(key, OverlayValue::Value(value));
    } else {
      self.base.insert(key, value);
    }
  }
  pub fn remove(&mut self, key: &K) {
    if self.overlay.is_some() {
      let overlay = self.overlay();
      overlay.insert(key.clone(), OverlayValue::Tombstone);
    } else {
      self.base.remove(key);
    }
  }

  pub fn get(&self, key: &K) -> Option<&V> {
    if let Some(overlay) = &self.overlay
      && let Some(value) = overlay.get(key)
    {
      return match value {
        OverlayValue::Value(value) => Some(value),
        OverlayValue::Tombstone => None,
      };
    }
    self.base.get(key)
  }

  /// Obtain a mutable reference. When overlay mode is on, the base entry is
  /// cloned into the overlay so mutations do not leak into the base map.
  pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
  where
    V: Clone,
  {
    if self.overlay.is_some() {
      self.materialize_overlay_value(key);
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      match overlay.get_mut(key) {
        Some(OverlayValue::Value(value)) => Some(value),
        _ => None,
      }
    } else {
      self.base.get_mut(key)
    }
  }

  pub fn iter(&self) -> OverlayIter<'_, K, V, S> {
    match &self.overlay {
      Some(overlay) => OverlayIter::Combined {
        overlay_keys: overlay,
        overlay_iter: overlay.iter(),
        base_iter: self.base.iter(),
      },
      None => OverlayIter::Base(self.base.iter()),
    }
  }

  pub fn has_overlay(&self) -> bool {
    self.overlay.is_some()
  }

  // if overlay enabled and key not in overlay, clone from base to overlay
  fn materialize_overlay_value(&mut self, key: &K)
  where
    V: Clone,
  {
    if self
      .overlay
      .as_ref()
      .expect("overlay checked above")
      .contains_key(key)
    {
      return;
    }

    if let Some(value) = self.base.get(key).cloned() {
      self
        .overlay()
        .insert(key.clone(), OverlayValue::Value(value));
    }
  }

  fn overlay(&mut self) -> &mut HashMap<K, OverlayValue<V>, S> {
    self.overlay.get_or_insert_with(HashMap::default)
  }
}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHashMap as HashMap;

  use super::OverlayMap;

  #[test]
  fn rebase_without_checkpoint_is_a_noop() {
    let mut map: OverlayMap<String, i32> = OverlayMap::default();

    map.rebase();

    // Recording must not be enabled by a rebase without a checkpoint:
    // otherwise writes would split between the base map and an overlay
    // that no reset will ever drop.
    assert!(!map.has_overlay());
    map.insert("a".to_string(), 1);
    assert!(!map.has_overlay());
    assert_eq!(map.get(&"a".to_string()), Some(&1));
  }

  #[test]
  fn rebase_folds_overlay_into_base_and_keeps_recording() {
    let mut map = OverlayMap::new(
      [("a".to_string(), 1)]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    );

    map.checkpoint();
    map.insert("b".to_string(), 2);
    map.remove(&"a".to_string());

    map.rebase();

    map.insert("c".to_string(), 3);

    assert!(map.has_overlay());
    assert_eq!(map.get(&"a".to_string()), None);
    assert_eq!(map.get(&"b".to_string()), Some(&2));
    assert_eq!(map.get(&"c".to_string()), Some(&3));

    map.reset();

    // Only the post-rebase change is discarded.
    assert_eq!(map.get(&"a".to_string()), None);
    assert_eq!(map.get(&"b".to_string()), Some(&2));
    assert_eq!(map.get(&"c".to_string()), None);
  }

  #[test]
  fn checkpoint_inserts_apply_only_to_overlay() {
    let mut map = OverlayMap::new(
      [("a".to_string(), 1)]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    );

    map.checkpoint();
    map.insert("b".to_string(), 2);
    map.insert("a".to_string(), 3);

    assert!(map.has_overlay());
    assert_eq!(map.get(&"a".to_string()), Some(&3));
    assert_eq!(map.get(&"b".to_string()), Some(&2));

    map.reset();

    assert!(!map.has_overlay());
    assert_eq!(map.get(&"a".to_string()), Some(&1));
    assert_eq!(map.get(&"b".to_string()), None);
  }

  #[test]
  fn remove_in_overlay_masks_base() {
    let mut map = OverlayMap::new(
      [("a".to_string(), 1), ("b".to_string(), 2)]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    );

    map.checkpoint();
    map.remove(&"a".to_string());

    assert_eq!(map.get(&"a".to_string()), None);
    assert_eq!(map.get(&"b".to_string()), Some(&2));

    map.reset();

    assert_eq!(map.get(&"a".to_string()), Some(&1));
    assert_eq!(map.get(&"b".to_string()), Some(&2));
  }

  #[test]
  fn get_mut_clones_base_into_overlay() {
    let mut map = OverlayMap::new(
      [("a".to_string(), 1)]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    );

    map.checkpoint();
    *map
      .get_mut(&"a".to_string())
      .expect("should clone base into overlay") = 5;

    assert_eq!(map.get(&"a".to_string()), Some(&5));

    map.reset();

    assert_eq!(map.get(&"a".to_string()), Some(&1));
  }
}
