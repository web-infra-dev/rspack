use rayon::prelude::*;

use crate::ChunkUkey;

/// Directly indexed values for one graph's current Chunk identities.
///
/// Like slotmap's SecondaryMap, each slot is either vacant or contains its
/// identity and payload. ChunkUkey's NonZeroU32 identity provides a niche for
/// Option, including for payloads with no niche of their own. Allocation and
/// recycling belong to ChunkGraph; the two tables cannot diverge in free lists.
/// Clone copies keys and values for graph snapshots, never the identity allocator.
#[derive(Debug, Clone)]
pub struct ChunkSlotMap<T> {
  slots: Vec<Option<(ChunkUkey, T)>>,
  len: usize,
}

impl<T> Default for ChunkSlotMap<T> {
  fn default() -> Self {
    Self::with_capacity(0)
  }
}

impl<T> ChunkSlotMap<T> {
  /// Reserve slots without initializing them. Capacity is a slot high-water
  /// mark, not a live count: deleted slots retain their index.
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      slots: Vec::with_capacity(capacity),
      len: 0,
    }
  }

  /// Number of addressable slots, including vacant ones.
  pub fn slot_count(&self) -> usize {
    self.slots.len()
  }

  #[inline]
  pub fn get(&self, key: &ChunkUkey) -> Option<&T> {
    let (stored, value) = self.slots.get(key.index())?.as_ref()?;
    (stored == key).then_some(value)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &ChunkUkey) -> Option<&mut T> {
    let (stored, value) = self.slots.get_mut(key.index())?.as_mut()?;
    (stored == key).then_some(value)
  }

  pub fn expect_get(&self, key: &ChunkUkey) -> &T {
    self
      .get(key)
      .unwrap_or_else(|| panic!("Chunk({key:?}) not found in ChunkSlotMap"))
  }

  pub fn expect_get_mut(&mut self, key: &ChunkUkey) -> &mut T {
    self
      .get_mut(key)
      .unwrap_or_else(|| panic!("Chunk({key:?}) not found in ChunkSlotMap"))
  }

  pub(super) fn insert(&mut self, key: ChunkUkey, value: T) -> Option<T> {
    let index = key.index();
    if index >= self.slots.len() {
      // Sequential insertion uses Vec's amortized growth. Only secondary maps
      // can skip indices; initialize just their missing prefix, never capacity.
      if index > self.slots.len() {
        self.slots.reserve(index + 1 - self.slots.len());
        self.slots.resize_with(index, || None);
      }
      self.slots.push(Some((key, value)));
      self.len += 1;
      return None;
    }
    let old = self.slots[index].replace((key, value));
    if old.is_none() {
      self.len += 1;
    }
    old
      .filter(|(stored, _)| *stored == key)
      .map(|(_, value)| value)
  }

  pub(super) fn remove(&mut self, key: &ChunkUkey) -> Option<T> {
    let slot = self.slots.get_mut(key.index())?;
    if slot.as_ref()?.0 != *key {
      return None;
    }
    let (_, value) = slot.take()?;
    self.len -= 1;
    Some(value)
  }

  #[inline]
  pub fn contains(&self, key: &ChunkUkey) -> bool {
    self.get(key).is_some()
  }
  pub fn len(&self) -> usize {
    self.len
  }
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }
  pub fn iter(&self) -> impl Iterator<Item = (&ChunkUkey, &T)> {
    self
      .slots
      .iter()
      .filter_map(|slot| slot.as_ref().map(|(key, value)| (key, value)))
  }
  pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ChunkUkey, &mut T)> {
    self
      .slots
      .iter_mut()
      .filter_map(|slot| slot.as_mut().map(|(key, value)| (&*key, value)))
  }
  pub(super) fn retain(&mut self, mut keep: impl FnMut(&ChunkUkey, &mut T) -> bool) {
    for slot in &mut self.slots {
      if let Some((key, value)) = slot
        && !keep(key, value)
      {
        let removed = slot.take();
        self.len -= 1;
        drop(removed);
      }
    }
  }

  pub(super) fn into_entries(self) -> impl Iterator<Item = (ChunkUkey, T)> {
    self.slots.into_iter().flatten()
  }

  pub(super) fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = (&ChunkUkey, &mut T)>
  where
    T: Send,
  {
    self
      .slots
      .par_iter_mut()
      .filter_map(|slot| slot.as_mut().map(|(key, value)| (&*key, value)))
  }

  pub fn keys(&self) -> impl Iterator<Item = &ChunkUkey> {
    self.iter().map(|(key, _)| key)
  }
  pub fn values(&self) -> impl Iterator<Item = &T> {
    self.iter().map(|(_, value)| value)
  }
  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.iter_mut().map(|(_, value)| value)
  }
}
