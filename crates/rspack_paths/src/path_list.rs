use std::{
  fmt,
  hash::{BuildHasher, Hash, Hasher},
  ops::Deref,
  sync::{
    Arc, OnceLock, Weak,
    atomic::{AtomicUsize, Ordering},
  },
};

use dashmap::{DashMap, mapref::entry::Entry};
#[cfg(feature = "cacheable")]
use rspack_cacheable::{
  ContextGuard, Result as CacheableResult, cacheable,
  with::{Custom, CustomConverter},
};
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::InternedPath;

/// A content-interned, immutable list of [`InternedPath`]s.
///
/// Lists with equal elements share one allocation process-wide, so a repeated
/// resolution result costs a handle instead of another element buffer.
/// Interning keys on the elements' precomputed path hashes, so a lookup hashes
/// list elements rather than path bytes; comparing a key hit compares elements
/// by their already interned identity. An empty list shares a single allocation.
///
/// The table holds weak references and a slot is removed as soon as its last
/// handle drops, so an entry lives exactly as long as some list still refers to
/// it and no sweeping is needed to reclaim a buffer whose list is gone.
/// Sparse shards shrink during removal, including releasing their bucket
/// allocation when empty, so the table does not retain its peak capacity.
#[cfg_attr(feature = "cacheable", cacheable(with = Custom))]
pub struct InternedPathList(Arc<ListStorage>);

/// The shared part of an interned list: its elements, the key that identifies
/// them in the intern table, and the number of handles that keep it alive.
struct ListStorage {
  /// Fold of the elements' precomputed path hashes: the hash half of the key.
  hash: u64,
  ids: Box<[InternedPath]>,
  /// Live handle count. Counting handles instead of inspecting the arc's
  /// strong count stays exact when several handles drop at once: the contained
  /// arc is only released after a handle's drop body returns.
  handles: AtomicUsize,
}

impl InternedPathList {
  /// Interns `items`, returning the shared list when an equal one exists.
  pub fn new(items: &[InternedPath]) -> Self {
    let table = intern_table();
    let key = list_key(items);
    match lookup(table, key, items) {
      Some(shared) => shared,
      None => intern(
        table,
        key,
        Arc::new(ListStorage {
          hash: key.0,
          ids: items.into(),
          handles: AtomicUsize::new(0),
        }),
      ),
    }
  }

  /// Interns an owned list, reusing its allocation when the list is new.
  pub fn from_vec(items: Vec<InternedPath>) -> Self {
    let table = intern_table();
    let key = list_key(&items);
    match lookup(table, key, &items) {
      Some(shared) => shared,
      None => intern(
        table,
        key,
        // Moving the vector hands its elements over to the shared allocation
        // instead of cloning and dropping every handle.
        Arc::new(ListStorage {
          hash: key.0,
          ids: items.into_boxed_slice(),
          handles: AtomicUsize::new(0),
        }),
      ),
    }
  }

  /// Counts a handle before releasing the table's shard lock, so a concurrent
  /// last drop cannot remove the slot between lookup and handle creation.
  /// Cloning needs no lock because the source handle keeps the list alive.
  fn from_storage(storage: Arc<ListStorage>) -> Self {
    storage.handles.fetch_add(1, Ordering::AcqRel);
    Self(storage)
  }

  #[inline]
  pub fn as_slice(&self) -> &[InternedPath] {
    &self.0.ids
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.0.ids.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.ids.is_empty()
  }
}

impl Clone for InternedPathList {
  fn clone(&self) -> Self {
    Self::from_storage(self.0.clone())
  }
}

impl Drop for InternedPathList {
  fn drop(&mut self) {
    // The last handle removes the slot: from here on the weak entry would keep
    // the buffer reserved for nobody.
    if self.0.handles.fetch_sub(1, Ordering::AcqRel) == 1 {
      remove_slot(&self.0);
    }
  }
}

impl Deref for InternedPathList {
  type Target = [InternedPath];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0.ids
  }
}

impl AsRef<[InternedPath]> for InternedPathList {
  #[inline]
  fn as_ref(&self) -> &[InternedPath] {
    &self.0.ids
  }
}

impl fmt::Debug for InternedPathList {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_list().entries(self.0.ids.iter()).finish()
  }
}

impl PartialEq for InternedPathList {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) || self.0.ids == other.0.ids
  }
}

impl Eq for InternedPathList {}

impl Hash for InternedPathList {
  /// Hashes by content like [`PartialEq`]: the interned fold of the elements
  /// plus the element count. Equal lists share the fold whether they are equal
  /// through their handle or through their elements, so hashing stays
  /// consistent with equality when a list keys a map.
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.0.hash);
    state.write_usize(self.0.ids.len());
  }
}

impl FromIterator<InternedPath> for InternedPathList {
  fn from_iter<T: IntoIterator<Item = InternedPath>>(iter: T) -> Self {
    Self::from_vec(iter.into_iter().collect())
  }
}

/// Intern table key: the fold of the elements' precomputed path hashes plus the
/// element count. A hit still compares elements, so a hash collision only costs
/// a duplicate entry, never a wrong list.
type ListKey = (u64, usize);

fn intern_table() -> &'static DashMap<ListKey, Weak<ListStorage>, FxBuildHasher> {
  static TABLE: OnceLock<DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>> = OnceLock::new();
  TABLE.get_or_init(|| DashMap::with_hasher(FxBuildHasher))
}

fn list_key(items: &[InternedPath]) -> ListKey {
  let mut hasher = FxHasher::default();
  for item in items {
    hasher.write_u64(item.precomputed_hash());
  }
  (hasher.finish(), items.len())
}

fn lookup(
  table: &DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>,
  key: ListKey,
  items: &[InternedPath],
) -> Option<InternedPathList> {
  let entry = table.get(&key)?;
  let existing = entry.value().upgrade()?;
  if existing.ids.len() == items.len() && existing.ids.iter().eq(items.iter()) {
    Some(InternedPathList::from_storage(existing))
  } else {
    None
  }
}

fn intern(
  table: &DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>,
  key: ListKey,
  list: Arc<ListStorage>,
) -> InternedPathList {
  // The write lock re-checks the entry so parallel interning of the same list
  // converges on one allocation instead of keeping one payload per thread.
  match table.entry(key) {
    Entry::Occupied(mut entry) => {
      let existing = entry.get().upgrade();
      match existing {
        Some(existing)
          if existing.ids.len() == list.ids.len() && existing.ids.iter().eq(list.ids.iter()) =>
        {
          InternedPathList::from_storage(existing)
        }
        Some(_) => {
          entry.insert(Arc::downgrade(&list));
          InternedPathList::from_storage(list)
        }
        None => {
          // The previous list with this content is gone; replacing the slot
          // releases its buffer right away.
          entry.insert(Arc::downgrade(&list));
          InternedPathList::from_storage(list)
        }
      }
    }
    Entry::Vacant(entry) => {
      // Inserting consumes the vacant entry and releases its lock. Count the
      // first handle before publishing the weak slot.
      let list = InternedPathList::from_storage(list);
      entry.insert(Arc::downgrade(&list.0));
      list
    }
  }
}

/// Removes the slot of the storage if it still belongs to it and no handle was
/// created meanwhile. A newer list with the same content may have replaced the
/// slot, and a handle may have been created while this drop was in flight; both
/// keep the slot in place.
fn remove_slot(storage: &Arc<ListStorage>) {
  let table = intern_table();
  let key = (storage.hash, storage.ids.len());
  let hash = table.hasher().hash_one(key);
  let mut shard = table.shards()[table.determine_shard(hash as usize)].write();
  if storage.handles.load(Ordering::Acquire) != 0 {
    return;
  }
  let removed = shard.remove_entry(hash, |(other_key, value)| {
    *other_key == key && value.get().as_ptr() == Arc::as_ptr(storage)
  });
  // Reuse the removal lock and visit only this shard. Leaving headroom avoids
  // repeatedly shrinking and growing when a workload hovers around a boundary.
  if removed.is_some() && shard.len() * 4 < shard.capacity() {
    let capacity = shard.len() * 2;
    shard.shrink_to(capacity, |(key, _)| table.hasher().hash_one(key));
  }
}

/// Persist the paths themselves; loading re-interns the list, so restored
/// builds share lists with any equal list already alive.
#[cfg(feature = "cacheable")]
impl CustomConverter for InternedPathList {
  type Target = Vec<InternedPath>;

  fn serialize(&self, _guard: &ContextGuard) -> CacheableResult<Self::Target> {
    Ok(self.0.ids.to_vec())
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> CacheableResult<Self> {
    Ok(Self::from_vec(data))
  }
}
