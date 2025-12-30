use std::{collections::HashMap, fmt::Debug, hash::Hash};

use rayon::iter::IntoParallelRefMutIterator as RayonIntoParallelRefMutIterator;
#[derive(Debug, Clone)]
pub enum Action<K, V> {
  Inserted { key: K, previous: Option<V> },
  Removed { key: K, value: V },
}

#[derive(Debug, Clone)]
pub struct RollbackMap<K, V> {
  map: HashMap<K, V>,
  undo_stack: Vec<Action<K, V>>,
  checkpoint: Option<usize>,
}

impl<K, V> Default for RollbackMap<K, V>
where
  K: Eq + Hash + Clone + Debug,
  V: Debug,
{
  fn default() -> Self {
    Self {
      map: Default::default(),
      undo_stack: Vec::new(),
      checkpoint: None,
    }
  }
}
impl<K, V> RollbackMap<K, V>
where
  K: Eq + Hash + Clone + Debug,
  V: Debug,
{
  /// Insert a key/value and record what was there before.
  pub fn insert(&mut self, key: K, value: V) -> bool {
    let previous = self.map.insert(key.clone(), value);
    let inserted = previous.is_none();
    if self.checkpoint.is_some() {
      self.undo_stack.push(Action::Inserted { key, previous });
    }
    inserted
  }

  /// Remove a key and record the removed value for undo.
  pub fn remove(&mut self, key: &K) -> bool {
    if let Some(value) = self.map.remove(key) {
      if self.checkpoint.is_some() {
        self.undo_stack.push(Action::Removed {
          key: key.clone(),
          value,
        });
      }
      true
    } else {
      false
    }
  }
  #[inline]
  pub fn get(&self, key: &K) -> Option<&V> {
    self.map.get(key)
  }
  #[inline]
  pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
    self.map.get_mut(key)
  }

  /// Start recording undo operations from this point to save memory.
  pub fn checkpoint(&mut self) {
    assert!(self.checkpoint.is_none());
    self.undo_stack.clear();
    self.checkpoint = Some(0);
  }

  /// Undo the most recent mutating operation. Returns true if something was undone.
  fn undo(&mut self) -> bool {
    let checkpoint = match self.checkpoint {
      Some(cp) => cp,
      None => return false,
    };

    if self.undo_stack.len() <= checkpoint {
      return false;
    }

    match self.undo_stack.pop() {
      Some(Action::Inserted { key, previous }) => match previous {
        Some(old_value) => {
          self.map.insert(key, old_value);
        }
        None => {
          self.map.remove(&key);
        }
      },
      Some(Action::Removed { key, value }) => {
        self.map.insert(key, value);
      }
      None => return false,
    }

    true
  }

  /// Undo everything recorded since the last checkpoint. Returns how many actions were undone.
  pub fn reset(&mut self) -> usize {
    let mut undone = 0;
    while self.undo() {
      undone += 1;
    }
    self.commit();
    undone
  }

  /// Clear the undo history and stop recording.
  fn commit(&mut self) {
    self.undo_stack.clear();
    self.checkpoint = None;
  }
  pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
    self.map.iter()
  }
  // check the length of mutations for debug performance purpose
  pub fn mutations_len(&self) -> usize {
    self.undo_stack.len()
  }
}

impl<'data, K, V> RayonIntoParallelRefMutIterator<'data> for RollbackMap<K, V>
where
  K: Eq + Hash + Send + Sync + 'data,
  V: Send + 'data,
{
  type Item = (&'data K, &'data mut V);
  type Iter = rayon::collections::hash_map::IterMut<'data, K, V>;

  fn par_iter_mut(&'data mut self) -> Self::Iter {
    self.map.par_iter_mut()
  }
}

#[cfg(test)]
mod tests {
  use super::RollbackMap;

  #[test]
  fn test_snapshot_map() {
    let mut snapshot_map: RollbackMap<String, i32> = RollbackMap::default();

    snapshot_map.insert("a".to_string(), 1);
    snapshot_map.insert("b".to_string(), 2);

    snapshot_map.checkpoint();

    snapshot_map.insert("c".to_string(), 3);
    snapshot_map.remove(&"a".to_string());

    assert_eq!(snapshot_map.get(&"a".to_string()), None);
    assert_eq!(snapshot_map.get(&"b".to_string()), Some(&2));
    assert_eq!(snapshot_map.get(&"c".to_string()), Some(&3));

    let undone = snapshot_map.reset();
    assert_eq!(undone, 2);

    assert_eq!(snapshot_map.get(&"a".to_string()), Some(&1));
    assert_eq!(snapshot_map.get(&"b".to_string()), Some(&2));
    assert_eq!(snapshot_map.get(&"c".to_string()), None);
  }
}
