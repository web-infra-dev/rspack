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

/// Lists built between two cheap checks of whether the intern table is worth
/// sweeping.
const SWEEP_CHECK_INTERVAL: usize = 1 << 10;

/// Entries below this count are never swept: the first sweep only starts once a
/// single pass is small relative to the entries that created the table.
const SWEEP_MIN_TABLE: usize = 1 << 16;

/// Floor of the observed dead-slot count that triggers a churn sweep.
const SWEEP_DEAD_SLOTS_MIN: usize = 1 << 10;

/// Lists built between two cheap checks of whether the intern table is worth
/// sweeping.
static SWEEP_TICKS: AtomicUsize = AtomicUsize::new(0);

/// Table size at which the next growth-driven sweep runs.
static NEXT_GROWTH_SWEEP: AtomicUsize = AtomicUsize::new(SWEEP_MIN_TABLE);

/// Lists whose last handle dropped since the last sweep.
static DEAD_SLOTS: AtomicUsize = AtomicUsize::new(0);

/// A content-interned, immutable list of [`InternedPath`]s.
///
/// Lists with equal elements share one allocation process-wide, so a repeated
/// resolution result costs one handle instead of another element buffer.
/// Interning keys on the elements' precomputed path hashes, so a lookup hashes
/// list elements rather than path bytes; comparing a key hit compares elements
/// by their already interned identity. An empty list shares a single allocation.
///
/// The table holds weak references and is swept when it grows or when enough
/// slots were observed to be dead, so an entry lives exactly as long as some
/// list still refers to it.
#[cfg_attr(feature = "cacheable", cacheable(with = Custom))]
#[derive(Clone)]
pub struct InternedPathList(Arc<[InternedPath]>);

impl InternedPathList {
  /// Interns `items`, returning the shared list when an equal one exists.
  pub fn new(items: &[InternedPath]) -> Self {
    let table = intern_table();
    let key = list_key(items);
    let list = match lookup(table, key, items) {
      Some(shared) => Self(shared),
      None => Self(intern(table, key, Arc::from(items))),
    };
    maybe_sweep(table);
    list
  }

  /// Interns an owned list, reusing its allocation when the list is new.
  pub fn from_vec(items: Vec<InternedPath>) -> Self {
    let table = intern_table();
    let key = list_key(&items);
    let list = match lookup(table, key, &items) {
      Some(shared) => Self(shared),
      // Moving the vector hands its elements over to the shared allocation
      // instead of cloning and dropping every handle.
      None => Self(intern(table, key, Arc::from(items))),
    };
    maybe_sweep(table);
    list
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

impl Drop for InternedPathList {
  fn drop(&mut self) {
    // The table only holds weak references, so this is the last handle: the
    // slot is dead from now on and its buffer stays reserved until a sweep
    // drops the weak entry. Counting the death here keeps sweeps driven by
    // real churn instead of by re-interning the same content, so lists that
    // never come back are still reclaimed. A concurrent `upgrade` can make the
    // count miss, which is fine: that other handle counts its own death, and
    // the growth trigger remains as a backstop.
    if Arc::strong_count(&self.0) == 1 {
      DEAD_SLOTS.fetch_add(1, Ordering::Relaxed);
    }
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
  // The write lock re-checks the entry so parallel interning of the same list
  // converges on one allocation instead of keeping one payload per thread.
  match table.entry(key) {
    Entry::Occupied(mut entry) => {
      let existing = entry.get().upgrade();
      match existing {
        Some(existing) if existing.len() == list.len() && existing.iter().eq(list.iter()) => {
          existing
        }
        Some(_) => {
          entry.insert(Arc::downgrade(&list));
          list
        }
        None => {
          // The previous list with this content is gone (its death was already
          // counted when its last handle dropped); replacing the slot releases
          // its buffer right away.
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

fn maybe_sweep(table: &DashMap<ListKey, Weak<[InternedPath]>, FxBuildHasher>) {
  if !SWEEP_TICKS
    .fetch_add(1, Ordering::Relaxed)
    .is_multiple_of(SWEEP_CHECK_INTERVAL)
  {
    return;
  }
  let entries = table.len();
  // Sweep after the table doubled since the previous sweep. This keeps the
  // initial build's total work at `O(entries log entries)` and acts as a
  // backstop for slots whose last handle dropped while a concurrent lookup held
  // a transient reference.
  let grown = entries >= NEXT_GROWTH_SWEEP.load(Ordering::Relaxed);
  // Sweep on churn as well: the slots of lists whose last handle dropped are
  // only reclaimed by a sweep, and a watch rebuild can drop many lists without
  // growing the table or re-interning the same content. Scale the trigger with
  // the table so one sweep stays proportional to what a rebuild can have
  // dropped.
  let churned = DEAD_SLOTS.load(Ordering::Relaxed) >= (entries / 8).max(SWEEP_DEAD_SLOTS_MIN);
  if !grown && !churned {
    return;
  }
  table.retain(|_, weak| weak.strong_count() > 0);
  DEAD_SLOTS.store(0, Ordering::Relaxed);
  NEXT_GROWTH_SWEEP.store(
    table.len().saturating_mul(2).max(SWEEP_MIN_TABLE),
    Ordering::Relaxed,
  );
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
