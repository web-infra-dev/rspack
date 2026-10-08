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

/// Insertions between two sweeps of dead interned lists.
const PRUNE_INTERVAL: usize = 1 << 15;

/// A content-interned, immutable list of [`InternedPath`]s.
///
/// Lists with equal elements share one allocation process-wide, so a repeated
/// resolution result costs one handle instead of another element buffer.
/// Interning keys on the elements' precomputed path hashes, so a lookup hashes
/// list elements rather than path bytes; comparing a key hit compares elements
/// by their already interned identity. An empty list shares a single allocation.
///
/// The table holds weak references and is swept periodically, so an entry lives
/// exactly as long as some list still refers to it.
#[cfg_attr(feature = "cacheable", cacheable(with = Custom))]
#[derive(Clone)]
pub struct InternedPathList(Arc<[InternedPath]>);

impl InternedPathList {
  /// Interns `items`, returning the shared list when an equal one exists.
  pub fn new(items: &[InternedPath]) -> Self {
    let table = intern_table();
    let key = list_key(items);
    if let Some(shared) = lookup(table, key, items) {
      return Self(shared);
    }
    Self(intern(table, key, Arc::from(items)))
  }

  /// Interns an owned list, reusing its allocation when the list is new.
  pub fn from_vec(items: Vec<InternedPath>) -> Self {
    let table = intern_table();
    let key = list_key(&items);
    if let Some(shared) = lookup(table, key, &items) {
      return Self(shared);
    }
    Self(intern(table, key, Arc::from(items.as_slice())))
  }

  #[inline]
  pub fn as_slice(&self) -> &[InternedPath] {
    &self.0
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

impl Deref for InternedPathList {
  type Target = [InternedPath];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<[InternedPath]> for InternedPathList {
  #[inline]
  fn as_ref(&self) -> &[InternedPath] {
    &self.0
  }
}

impl fmt::Debug for InternedPathList {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_list().entries(self.0.iter()).finish()
  }
}

impl PartialEq for InternedPathList {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) || self.0[..] == other.0[..]
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

fn intern_table() -> &'static DashMap<ListKey, Weak<[InternedPath]>, FxBuildHasher> {
  static TABLE: OnceLock<DashMap<ListKey, Weak<[InternedPath]>, FxBuildHasher>> = OnceLock::new();
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
  table: &DashMap<ListKey, Weak<[InternedPath]>, FxBuildHasher>,
  key: ListKey,
  items: &[InternedPath],
) -> Option<Arc<[InternedPath]>> {
  let existing = table.get(&key)?.value().upgrade()?;
  if existing.len() == items.len() && existing.iter().eq(items.iter()) {
    Some(existing)
  } else {
    None
  }
}

fn intern(
  table: &DashMap<ListKey, Weak<[InternedPath]>, FxBuildHasher>,
  key: ListKey,
  list: Arc<[InternedPath]>,
) -> Arc<[InternedPath]> {
  let shared = match table.entry(key) {
    // The write lock re-checks the entry so parallel interning of the same list
    // converges on one allocation instead of keeping one payload per thread.
    Entry::Occupied(mut entry) => {
      let existing = entry.get().upgrade();
      match existing {
        Some(existing) if existing.len() == list.len() && existing.iter().eq(list.iter()) => {
          existing
        }
        _ => {
          entry.insert(Arc::downgrade(&list));
          list
        }
      }
    }
    Entry::Vacant(entry) => {
      entry.insert(Arc::downgrade(&list));
      list
    }
  };
  prune(table);
  shared
}

fn prune(table: &DashMap<ListKey, Weak<[InternedPath]>, FxBuildHasher>) {
  static INSERTS: AtomicUsize = AtomicUsize::new(0);
  if INSERTS.fetch_add(1, Ordering::Relaxed) + 1 < PRUNE_INTERVAL {
    return;
  }
  INSERTS.store(0, Ordering::Relaxed);
  table.retain(|_, weak| weak.strong_count() > 0);
}

/// Persist the paths themselves; loading re-interns the list, so restored
/// builds share lists with any equal list already alive.
#[cfg(feature = "cacheable")]
impl CustomConverter for InternedPathList {
  type Target = Vec<InternedPath>;

  fn serialize(&self, _guard: &ContextGuard) -> CacheableResult<Self::Target> {
    Ok(self.0.to_vec())
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> CacheableResult<Self> {
    Ok(Self::from_vec(data))
  }
}
