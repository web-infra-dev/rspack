use std::{
  fmt,
  hash::Hasher,
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
      Some(shared) => Self::from_storage(shared),
      None => Self::from_storage(intern(
        table,
        key,
        Arc::new(ListStorage {
          hash: key.0,
          ids: items.into(),
          handles: AtomicUsize::new(0),
        }),
      )),
    }
  }

  /// Interns an owned list, reusing its allocation when the list is new.
  pub fn from_vec(items: Vec<InternedPath>) -> Self {
    let table = intern_table();
    let key = list_key(&items);
    match lookup(table, key, &items) {
      Some(shared) => Self::from_storage(shared),
      None => Self::from_storage(intern(
        table,
        key,
        // Moving the vector hands its elements over to the shared allocation
        // instead of cloning and dropping every handle.
        Arc::new(ListStorage {
          hash: key.0,
          ids: items.into_boxed_slice(),
          handles: AtomicUsize::new(0),
        }),
      )),
    }
  }

  /// Wraps a shared list in a new handle. The first handle makes sure the slot
  /// exists: the previous last drop may have removed it already, and a live
  /// handle must not keep its list out of the table.
  fn from_storage(storage: Arc<ListStorage>) -> Self {
    if storage.handles.fetch_add(1, Ordering::AcqRel) == 0 {
      register_slot(&storage);
    }
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
) -> Option<Arc<ListStorage>> {
  let existing = table.get(&key)?.value().upgrade()?;
  if existing.ids.len() == items.len() && existing.ids.iter().eq(items.iter()) {
    Some(existing)
  } else {
    None
  }
}

fn intern(
  table: &DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>,
  key: ListKey,
  list: Arc<ListStorage>,
) -> Arc<ListStorage> {
  // The write lock re-checks the entry so parallel interning of the same list
  // converges on one allocation instead of keeping one payload per thread.
  match table.entry(key) {
    Entry::Occupied(mut entry) => {
      let existing = entry.get().upgrade();
      match existing {
        Some(existing)
          if existing.ids.len() == list.ids.len() && existing.ids.iter().eq(list.ids.iter()) =>
        {
          existing
        }
        Some(_) => {
          entry.insert(Arc::downgrade(&list));
          list
        }
        None => {
          // The previous list with this content is gone; replacing the slot
          // releases its buffer right away.
          entry.insert(Arc::downgrade(&list));
          list
        }
      }
    }
    Entry::Vacant(entry) => {
      entry.insert(Arc::downgrade(&list));
      list
    }
  }
}

/// Makes sure some equal list occupies the slot of the storage. Runs under the
/// entry lock, which serialises it against remove_slot: either it runs after
/// the removal and re-inserts the slot, or the removal re-checks the handle
/// count and leaves the slot in place.
fn register_slot(storage: &Arc<ListStorage>) {
  let table = intern_table();
  let key = (storage.hash, storage.ids.len());
  match table.entry(key) {
    Entry::Occupied(mut entry) => {
      // An equal live list is already interned; nothing to insert.
      if entry.get().upgrade().is_none() {
        entry.insert(Arc::downgrade(storage));
      }
    }
    Entry::Vacant(entry) => {
      entry.insert(Arc::downgrade(storage));
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
  if let Entry::Occupied(entry) = table.entry(key) {
    if storage.handles.load(Ordering::Acquire) != 0 {
      return;
    }
    let points_at_self = match entry.get().upgrade() {
      Some(existing) => Arc::ptr_eq(&existing, storage),
      // The slot is dead and can only hold a list that is on its way out;
      // removing it releases the buffer together with the weak entry.
      None => true,
    };
    if points_at_self {
      entry.remove();
    }
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
