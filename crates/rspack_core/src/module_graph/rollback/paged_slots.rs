use std::fmt;

const PAGE_BITS: usize = 10;
const PAGE_SIZE: usize = 1 << PAGE_BITS;
const PAGE_MASK: usize = PAGE_SIZE - 1;
// Lets slightly out-of-order inserts during a build stay in the flat prefix.
const FLAT_SLACK: usize = 16 * PAGE_SIZE;

#[derive(Debug, Clone, Copy, Default)]
pub struct DenseStorageStats {
  pub id_space: usize,
  pub live: usize,
  /// Live entries stored in pages instead of the flat prefix.
  pub paged: usize,
  pub slots: usize,
  /// Bytes held by the table itself, not by heap data the values own.
  pub bytes: usize,
}

struct Page<V> {
  live: usize,
  slots: Box<[Option<V>; PAGE_SIZE]>,
}

impl<V> Page<V> {
  fn new() -> Self {
    let slots: Box<[Option<V>]> = std::iter::repeat_with(|| None).take(PAGE_SIZE).collect();
    Self {
      live: 0,
      slots: slots
        .try_into()
        .unwrap_or_else(|_| unreachable!("page is allocated with PAGE_SIZE slots")),
    }
  }
}

/// Storage indexed by ids that are never reused, so that its size follows the
/// number of live entries instead of the number of ids ever allocated.
///
/// Ids inserted densely before the first removal live in a flat prefix, which
/// covers the initial build. Later ids go into pages that are freed once empty.
pub struct PagedSlots<V> {
  /// Its length is a multiple of `PAGE_SIZE`.
  flat: Vec<Option<V>>,
  flat_live: usize,
  flat_frozen: bool,
  /// Only holds ids at or above `flat.len()`.
  pages: Vec<Option<Page<V>>>,
  paged_live: usize,
  end: usize,
}

impl<V> Default for PagedSlots<V> {
  fn default() -> Self {
    Self {
      flat: Vec::new(),
      flat_live: 0,
      flat_frozen: false,
      pages: Vec::new(),
      paged_live: 0,
      end: 0,
    }
  }
}

impl<V: fmt::Debug> fmt::Debug for PagedSlots<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_map().entries(self.iter()).finish()
  }
}

impl<V> PagedSlots<V> {
  /// One past the largest inserted index. Removals do not lower it.
  #[inline]
  pub fn end(&self) -> usize {
    self.end
  }

  #[inline]
  pub fn get(&self, index: usize) -> Option<&V> {
    match self.flat.get(index) {
      Some(slot) => slot.as_ref(),
      None => self.get_paged(index),
    }
  }

  #[inline]
  pub fn get_mut(&mut self, index: usize) -> Option<&mut V> {
    if index < self.flat.len() {
      self.flat[index].as_mut()
    } else {
      self.get_paged_mut(index)
    }
  }

  #[inline]
  pub fn contains(&self, index: usize) -> bool {
    self.get(index).is_some()
  }

  #[inline]
  pub fn insert(&mut self, index: usize, value: V) -> Option<V> {
    self.end = self.end.max(index + 1);
    if index >= self.flat.len() && !self.try_grow_flat(index) {
      return self.insert_paged(index, value);
    }
    let previous = self.flat[index].replace(value);
    if previous.is_none() {
      self.flat_live += 1;
    }
    previous
  }

  #[inline]
  pub fn remove(&mut self, index: usize) -> Option<V> {
    if let Some(slot) = self.flat.get_mut(index) {
      let value = slot.take()?;
      self.flat_live -= 1;
      self.on_flat_remove();
      return Some(value);
    }
    let entry = self.pages.get_mut(index >> PAGE_BITS)?;
    let page = entry.as_mut()?;
    let value = page.slots[index & PAGE_MASK].take()?;
    page.live -= 1;
    self.paged_live -= 1;
    if page.live == 0 {
      *entry = None;
    }
    Some(value)
  }

  #[inline]
  pub fn clear(&mut self) {
    *self = Self::default();
  }

  /// Yields entries in ascending index order.
  pub fn iter(&self) -> impl Iterator<Item = (usize, &V)> {
    let flat = self
      .flat
      .iter()
      .enumerate()
      .filter_map(|(index, value)| Some((index, value.as_ref()?)));
    let paged = self
      .pages
      .iter()
      .enumerate()
      .filter_map(|(page_index, page)| Some((page_index << PAGE_BITS, page.as_ref()?)))
      .flat_map(|(page_start, page)| {
        page
          .slots
          .iter()
          .enumerate()
          .filter_map(move |(offset, value)| Some((page_start + offset, value.as_ref()?)))
      });
    flat.chain(paged)
  }

  pub fn stats(&self) -> DenseStorageStats {
    let pages = self.pages.iter().flatten().count();
    DenseStorageStats {
      id_space: self.end,
      live: self.flat_live + self.paged_live,
      paged: self.paged_live,
      slots: self.flat.capacity() + pages * PAGE_SIZE,
      bytes: self.flat.capacity() * std::mem::size_of::<Option<V>>()
        + self.pages.capacity() * std::mem::size_of::<Option<Page<V>>>()
        + pages * std::mem::size_of::<[Option<V>; PAGE_SIZE]>(),
    }
  }

  #[cold]
  fn try_grow_flat(&mut self, index: usize) -> bool {
    if self.flat_frozen {
      return false;
    }
    let new_len = (index + 1).next_multiple_of(PAGE_SIZE);
    let live = self.flat_live + self.paged_live + 1;
    if new_len > live.saturating_mul(2).saturating_add(FLAT_SLACK) {
      return false;
    }

    let old_page_end = self.flat.len() >> PAGE_BITS;
    self.flat.resize_with(new_len, || None);
    let new_page_end = (new_len >> PAGE_BITS).min(self.pages.len());
    for page_index in old_page_end..new_page_end {
      let Some(page) = self.pages[page_index].take() else {
        continue;
      };
      self.paged_live -= page.live;
      self.flat_live += page.live;
      let page_start = page_index << PAGE_BITS;
      let slots: Box<[Option<V>]> = page.slots;
      for (offset, value) in slots.into_vec().into_iter().enumerate() {
        if value.is_some() {
          self.flat[page_start + offset] = value;
        }
      }
    }
    true
  }

  #[inline]
  fn get_paged(&self, index: usize) -> Option<&V> {
    self.pages.get(index >> PAGE_BITS)?.as_ref()?.slots[index & PAGE_MASK].as_ref()
  }

  #[inline]
  fn get_paged_mut(&mut self, index: usize) -> Option<&mut V> {
    self.pages.get_mut(index >> PAGE_BITS)?.as_mut()?.slots[index & PAGE_MASK].as_mut()
  }

  #[inline]
  fn insert_paged(&mut self, index: usize, value: V) -> Option<V> {
    let page_index = index >> PAGE_BITS;
    if self.pages.len() <= page_index {
      self.pages.resize_with(page_index + 1, || None);
    }
    let page = self.pages[page_index].get_or_insert_with(Page::new);
    let previous = page.slots[index & PAGE_MASK].replace(value);
    if previous.is_none() {
      page.live += 1;
      self.paged_live += 1;
    }
    previous
  }

  #[inline]
  fn on_flat_remove(&mut self) {
    if !self.flat_frozen {
      // Removed ids never come back, so the prefix will not get denser.
      self.flat_frozen = true;
      self.flat.shrink_to_fit();
    }
    if self.flat.len() >= FLAT_SLACK && self.flat_live * 3 < self.flat.len() {
      self.demote_flat();
    }
  }

  #[cold]
  fn demote_flat(&mut self) {
    let flat = std::mem::take(&mut self.flat);
    self.flat_live = 0;
    for (index, value) in flat.into_iter().enumerate() {
      if let Some(value) = value {
        self.insert_paged(index, value);
      }
    }
  }
}
