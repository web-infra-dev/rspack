use std::{collections::hash_map::Iter as HashMapIter, fmt::Debug, hash::Hash};

use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Clone)]
pub enum OverlayValue<V> {
  Value(V),
  Tombstone,
}

#[derive(Debug, Clone)]
pub struct OverlayMap<K, V> {
  base: HashMap<K, V>,
  overlay: Option<HashMap<K, OverlayValue<V>>>,
}

impl<K, V> Default for OverlayMap<K, V>
where
  K: Eq + Hash,
{
  fn default() -> Self {
    Self {
      base: HashMap::default(),
      overlay: None,
    }
  }
}

#[derive(Debug, Clone)]
pub enum OverlayIter<'a, K, V> {
  Base(HashMapIter<'a, K, V>),
  Combined {
    overlay_keys: &'a HashMap<K, OverlayValue<V>>,
    overlay_iter: HashMapIter<'a, K, OverlayValue<V>>,
    base_iter: HashMapIter<'a, K, V>,
  },
}

impl<'a, K, V> Iterator for OverlayIter<'a, K, V>
where
  K: Eq + Hash,
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

impl<K, V> OverlayMap<K, V>
where
  K: Eq + Hash + Clone,
{
  pub fn new(base: HashMap<K, V>) -> Self {
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

  pub fn iter(&self) -> OverlayIter<'_, K, V> {
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

  fn overlay(&mut self) -> &mut HashMap<K, OverlayValue<V>> {
    self.overlay.get_or_insert_with(HashMap::default)
  }
}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHashMap as HashMap;

  use super::OverlayMap;

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
