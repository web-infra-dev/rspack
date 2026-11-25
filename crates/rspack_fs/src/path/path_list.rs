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
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::AbsPath;

/// A content-interned, immutable list of [`AbsPath`]s.
///
/// Lists with equal elements share one allocation process-wide and are reclaimed
/// as soon as the last handle drops.
pub struct AbsPathList(Arc<ListStorage>);

struct ListStorage {
  hash: u64,
  ids: Box<[AbsPath]>,
  handles: AtomicUsize,
}

impl AbsPathList {
  pub fn new(items: &[AbsPath]) -> Self {
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

  pub fn from_vec(items: Vec<AbsPath>) -> Self {
    let table = intern_table();
    let key = list_key(&items);
    match lookup(table, key, &items) {
      Some(shared) => shared,
      None => intern(
        table,
        key,
        Arc::new(ListStorage {
          hash: key.0,
          ids: items.into_boxed_slice(),
          handles: AtomicUsize::new(0),
        }),
      ),
    }
  }

  fn from_storage(storage: Arc<ListStorage>) -> Self {
    storage.handles.fetch_add(1, Ordering::AcqRel);
    Self(storage)
  }

  #[inline]
  pub fn as_slice(&self) -> &[AbsPath] {
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

impl Default for AbsPathList {
  fn default() -> Self {
    Self::new(&[])
  }
}

impl Clone for AbsPathList {
  fn clone(&self) -> Self {
    Self::from_storage(self.0.clone())
  }
}

impl Drop for AbsPathList {
  fn drop(&mut self) {
    if self.0.handles.fetch_sub(1, Ordering::AcqRel) == 1 {
      remove_slot(&self.0);
    }
  }
}

impl Deref for AbsPathList {
  type Target = [AbsPath];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0.ids
  }
}

impl AsRef<[AbsPath]> for AbsPathList {
  #[inline]
  fn as_ref(&self) -> &[AbsPath] {
    &self.0.ids
  }
}

impl fmt::Debug for AbsPathList {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_list().entries(self.0.ids.iter()).finish()
  }
}

impl PartialEq for AbsPathList {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) || (self.0.hash == other.0.hash && self.0.ids == other.0.ids)
  }
}

impl Eq for AbsPathList {}

impl Hash for AbsPathList {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.0.hash);
    state.write_usize(self.0.ids.len());
  }
}

impl FromIterator<AbsPath> for AbsPathList {
  fn from_iter<T: IntoIterator<Item = AbsPath>>(iter: T) -> Self {
    Self::from_vec(iter.into_iter().collect())
  }
}

type ListKey = (u64, usize);

fn intern_table() -> &'static DashMap<ListKey, Weak<ListStorage>, FxBuildHasher> {
  static TABLE: OnceLock<DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>> = OnceLock::new();
  TABLE.get_or_init(|| DashMap::with_hasher(FxBuildHasher))
}

fn list_key(items: &[AbsPath]) -> ListKey {
  let mut hasher = FxHasher::default();
  for item in items {
    hasher.write_u64(item.precomputed_hash());
  }
  (hasher.finish(), items.len())
}

fn lookup(
  table: &DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>,
  key: ListKey,
  items: &[AbsPath],
) -> Option<AbsPathList> {
  let entry = table.get(&key)?;
  let existing = entry.value().upgrade()?;
  if existing.ids.as_ref() == items {
    Some(AbsPathList::from_storage(existing))
  } else {
    None
  }
}

fn intern(
  table: &DashMap<ListKey, Weak<ListStorage>, FxBuildHasher>,
  key: ListKey,
  list: Arc<ListStorage>,
) -> AbsPathList {
  match table.entry(key) {
    Entry::Occupied(mut entry) => {
      let existing = entry.get().upgrade();
      match existing {
        Some(existing) if existing.ids == list.ids => AbsPathList::from_storage(existing),
        Some(_) | None => {
          entry.insert(Arc::downgrade(&list));
          AbsPathList::from_storage(list)
        }
      }
    }
    Entry::Vacant(entry) => {
      let list = AbsPathList::from_storage(list);
      entry.insert(Arc::downgrade(&list.0));
      list
    }
  }
}

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
  if removed.is_some() && shard.len() * 4 < shard.capacity() {
    let capacity = shard.len() * 2;
    shard.shrink_to(capacity, |(key, _)| table.hasher().hash_one(key));
  }
}
