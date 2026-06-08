use std::fmt::Debug;

use rayon::prelude::*;
use rspack_collections::IdentifierMap;

use crate::{ModuleId, ModuleIdentifier};

#[derive(Debug, Clone)]
struct DenseVecEntry<V> {
  identifier: ModuleIdentifier,
  value: V,
}

#[derive(Debug, Clone)]
enum Action<V> {
  Inserted {
    id: ModuleId,
    identifier: ModuleIdentifier,
    previous: Option<V>,
  },
  Removed {
    id: ModuleId,
    identifier: ModuleIdentifier,
    value: V,
  },
}

/// Dense storage keyed by an internal numeric `ModuleId`.
///
/// External callers still address modules by `ModuleIdentifier`; this map keeps
/// a side index from identifier to the dense `ModuleId` slot, while the module
/// values themselves live in a dense vector. This mirrors esbuild's pattern of
/// storing modules in a vector and using a compact id as the primary key.
#[derive(Debug, Clone)]
pub struct DenseVec<V> {
  values: Vec<Option<DenseVecEntry<V>>>,
  identifiers: IdentifierMap<ModuleId>,
  undo_stack: Vec<Action<V>>,
  checkpoint: Option<usize>,
  len: usize,
}

impl<V> Default for DenseVec<V> {
  fn default() -> Self {
    Self {
      values: Vec::new(),
      identifiers: IdentifierMap::default(),
      undo_stack: Vec::new(),
      checkpoint: None,
      len: 0,
    }
  }
}

impl<V> DenseVec<V>
where
  V: Debug,
{
  #[inline]
  pub fn insert(&mut self, identifier: ModuleIdentifier, value: V) -> bool {
    let id = self.get_or_insert_id(identifier);
    let index = module_id_to_index(&id);
    if let Some(entry) = self.values[index].as_mut() {
      let previous = std::mem::replace(&mut entry.value, value);
      if self.checkpoint.is_some() {
        self.undo_stack.push(Action::Inserted {
          id,
          identifier,
          previous: Some(previous),
        });
      }
      false
    } else {
      self.values[index] = Some(DenseVecEntry { identifier, value });
      self.len += 1;
      if self.checkpoint.is_some() {
        self.undo_stack.push(Action::Inserted {
          id,
          identifier,
          previous: None,
        });
      }
      true
    }
  }

  #[inline]
  pub fn get_id(&self, identifier: &ModuleIdentifier) -> Option<&ModuleId> {
    self.identifiers.get(identifier)
  }

  #[inline]
  pub fn get_or_insert_id(&mut self, identifier: ModuleIdentifier) -> ModuleId {
    if let Some(id) = self.identifiers.get(&identifier) {
      return id.clone();
    }
    let id = ModuleId::from(self.values.len() as u32);
    self.values.push(None);
    self.identifiers.insert(identifier, id.clone());
    id
  }

  #[inline]
  pub fn get_by_id(&self, id: &ModuleId) -> Option<&V> {
    self
      .values
      .get(module_id_to_index(id))?
      .as_ref()
      .map(|entry| &entry.value)
  }

  #[inline]
  pub fn get_mut_by_id(&mut self, id: &ModuleId) -> Option<&mut V> {
    self
      .values
      .get_mut(module_id_to_index(id))?
      .as_mut()
      .map(|entry| &mut entry.value)
  }

  #[inline]
  pub fn remove(&mut self, identifier: &ModuleIdentifier) -> bool {
    let Some(id) = self.identifiers.remove(identifier) else {
      return false;
    };
    let index = module_id_to_index(&id);
    let entry = self.values[index]
      .take()
      .expect("identifier index should point to an occupied module slot");
    self.len -= 1;
    if self.checkpoint.is_some() {
      self.undo_stack.push(Action::Removed {
        id,
        identifier: entry.identifier,
        value: entry.value,
      });
    }
    true
  }

  #[inline]
  pub fn get(&self, identifier: &ModuleIdentifier) -> Option<&V> {
    let id = self.identifiers.get(identifier)?;
    self
      .values
      .get(module_id_to_index(id))?
      .as_ref()
      .map(|entry| &entry.value)
  }

  #[inline]
  pub fn get_mut(&mut self, identifier: &ModuleIdentifier) -> Option<&mut V> {
    let id = self.identifiers.get(identifier)?;
    self
      .values
      .get_mut(module_id_to_index(id))?
      .as_mut()
      .map(|entry| &mut entry.value)
  }

  #[inline]
  pub fn checkpoint(&mut self) {
    assert!(self.checkpoint.is_none());
    self.undo_stack.clear();
    self.checkpoint = Some(0);
  }

  pub fn reset(&mut self) -> usize {
    let checkpoint = match self.checkpoint {
      Some(checkpoint) => checkpoint,
      None => return 0,
    };

    let mut undone = 0;
    while self.undo_stack.len() > checkpoint {
      undone += 1;
      match self.undo_stack.pop() {
        Some(Action::Inserted {
          id,
          identifier,
          previous,
        }) => {
          let index = module_id_to_index(&id);
          match previous {
            Some(previous) => {
              self.values[index] = Some(DenseVecEntry {
                identifier,
                value: previous,
              });
              self.identifiers.insert(identifier, id);
            }
            None => {
              if self.values[index].take().is_some() {
                self.len -= 1;
              }
              self.identifiers.remove(&identifier);
            }
          }
        }
        Some(Action::Removed {
          id,
          identifier,
          value,
        }) => {
          let index = module_id_to_index(&id);
          self.values[index] = Some(DenseVecEntry { identifier, value });
          self.identifiers.insert(identifier, id);
          self.len += 1;
        }
        None => break,
      }
    }

    self.undo_stack.clear();
    self.checkpoint = None;
    undone
  }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (&ModuleIdentifier, &V)> {
    self.values.iter().filter_map(|entry| {
      entry
        .as_ref()
        .map(|entry| (&entry.identifier, &entry.value))
    })
  }

  #[inline]
  pub fn par_iter(&self) -> impl ParallelIterator<Item = (&ModuleIdentifier, &V)>
  where
    V: Sync,
  {
    self.values.par_iter().filter_map(|entry| {
      entry
        .as_ref()
        .map(|entry| (&entry.identifier, &entry.value))
    })
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.len
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }
}

#[inline]
fn module_id_to_index(id: &ModuleId) -> usize {
  id.as_number()
    .expect("dense module ids should always be numeric") as usize
}

#[cfg(test)]
mod tests {
  use super::DenseVec;
  use crate::ModuleIdentifier;

  #[test]
  fn supports_sparse_removal_without_reusing_ids() {
    let mut map = DenseVec::default();
    let a = ModuleIdentifier::from("a");
    let b = ModuleIdentifier::from("b");

    assert!(map.insert(a, 1));
    assert!(map.insert(b, 2));
    assert!(map.remove(&a));

    assert_eq!(map.get(&a), None);
    assert_eq!(map.get(&b), Some(&2));
    assert_eq!(map.iter().collect::<Vec<_>>(), vec![(&b, &2)]);
  }

  #[test]
  fn reset_restores_inserts_updates_and_removes() {
    let mut map = DenseVec::default();
    let a = ModuleIdentifier::from("a");
    let b = ModuleIdentifier::from("b");

    map.insert(a, 1);
    map.checkpoint();
    map.insert(a, 3);
    map.insert(b, 2);
    map.remove(&a);

    assert_eq!(map.get(&a), None);
    assert_eq!(map.get(&b), Some(&2));

    assert_eq!(map.reset(), 3);

    assert_eq!(map.get(&a), Some(&1));
    assert_eq!(map.get(&b), None);
  }
}
