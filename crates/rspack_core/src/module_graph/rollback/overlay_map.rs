use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[derive(Debug, Clone)]
pub struct OverlayMap<K, V> {
  base: HashMap<K, V>,
  pub overlay: Option<HashMap<K, V>>,
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
    if self.overlay.is_none() {
      self.overlay = Some(HashMap::default());
    }
  }

  /// Drop the overlay without applying its changes.
  pub fn recover_from_last_checkpoint(&mut self) {
    self.overlay = None;
  }

  pub fn contains_key(&self, key: &K) -> bool {
    self.get(key).is_some()
  }

  pub fn insert(&mut self, key: K, value: V) {
    if self.overlay.is_some() {
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      overlay.insert(key, value);
    } else {
      self.base.insert(key, value);
    }
  }

  pub fn remove(&mut self, key: &K) {
    if self.overlay.is_some() {
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      overlay.remove(key);
    } else {
      self.base.remove(key);
    }
  }

  pub fn get(&self, key: &K) -> Option<&V> {
    if let Some(overlay) = &self.overlay
      && let Some(value) = overlay.get(key)
    {
      return Some(value);
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
        Some(value) => Some(value),
        None => None,
      }
    } else {
      self.base.get_mut(key)
    }
  }
  // if overlay enabled and key not in overlay, clone from base to overlay
  fn materialize_overlay_value(&mut self, key: &K)
  where
    V: Clone,
  {
    let should_materialize = {
      let overlay = self.overlay.as_ref().expect("overlay checked above");
      !overlay.contains_key(key)
    };

    if should_materialize && let Some(value) = self.base.get(key).cloned() {
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      overlay.insert(key.clone(), value);
    }
  }
  pub fn has_overlay(&self) -> bool {
    self.overlay.is_some()
  }
}
