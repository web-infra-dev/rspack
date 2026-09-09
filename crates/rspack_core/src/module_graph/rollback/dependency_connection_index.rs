use std::num::NonZeroU32;

use crate::{DependencyId, ModuleGraphConnectionId};

/// The primary connection for each dependency, stored in four-byte slots.
/// Encodes connection IDs as `id + 1`, leaving zero for missing connections.
/// Writes record their previous values after a checkpoint so reads only need
/// one array lookup, without checking a separate overlay.
#[derive(Debug, Default)]
pub struct DependencyConnectionIndex {
  values: Vec<Option<NonZeroU32>>,
  undo: Vec<(DependencyId, Option<NonZeroU32>)>,
  checkpoint_len: Option<usize>,
}

impl DependencyConnectionIndex {
  #[inline]
  pub fn get(&self, dependency_id: &DependencyId) -> Option<ModuleGraphConnectionId> {
    self
      .values
      .get(dependency_id.as_u32() as usize)
      .copied()
      .flatten()
      .map(|value| ModuleGraphConnectionId::from(value.get() - 1))
  }

  #[inline]
  pub fn insert(&mut self, dependency_id: DependencyId, connection_id: ModuleGraphConnectionId) {
    let value = NonZeroU32::new(
      connection_id
        .checked_add(1)
        .expect("too many module graph connections"),
    );
    let index = dependency_id.as_u32() as usize;
    if self.values.len() <= index {
      self.values.resize(index + 1, None);
    }
    if let Some(len) = self.checkpoint_len
      && index < len
    {
      self.undo.push((dependency_id, self.values[index]));
    }
    self.values[index] = value;
  }

  pub fn remove(&mut self, dependency_id: &DependencyId) {
    if let Some(value) = self.values.get_mut(dependency_id.as_u32() as usize)
      && let Some(previous) = value.take()
    {
      self.record(*dependency_id, Some(previous));
    }
  }

  fn record(&mut self, dependency_id: DependencyId, previous: Option<NonZeroU32>) {
    if let Some(len) = self.checkpoint_len
      && (dependency_id.as_u32() as usize) < len
    {
      self.undo.push((dependency_id, previous));
    }
  }

  pub fn checkpoint(&mut self) {
    self.checkpoint_len.get_or_insert(self.values.len());
  }

  pub fn reset(&mut self) {
    if let Some(len) = self.checkpoint_len.take() {
      for (dependency_id, previous) in self.undo.drain(..).rev() {
        self.values[dependency_id.as_u32() as usize] = previous;
      }
      self.values.truncate(len);
    }
  }
}
