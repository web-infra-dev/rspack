use std::{
  fmt,
  hash::{Hash, Hasher},
  sync::{
    Arc,
    atomic::{AtomicU32, Ordering, fence},
  },
};

use dashmap::mapref::entry::Entry;
use rspack_paths::{InternedPath, InternedPathDashMap};
use sharded_slab::Slab;

/// An opaque sharded-slab key, meaningful only inside its owning FileTable.
///
/// The default slab configuration uses all usize bits, including its generation
/// counter. Copying a key does not retain the entry, so lookup and retain by a
/// bare FileId are not exposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct FileId(usize);

struct FileEntry {
  path: InternedPath,
  owners: AtomicU32,
}

impl FileEntry {
  fn retain(&self) {
    // The caller already owns a reference or holds the path's index guard.
    self
      .owners
      .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |owners| {
        owners.checked_add(1)
      })
      .expect("filesystem entry reference count overflow");
  }
}

/// File identities for one filesystem namespace.
///
/// Share an Arc to this table to share a namespace. A separately constructed
/// table is a separate namespace, even when paths have the same spelling.
/// The caller associates the table with the filesystem; the table itself must
/// not retain a filesystem, compiler, logger, or observation runtime.
///
/// Intern and final release synchronize through the path's DashMap shard.
/// Clone, path access, and non-final releases use slab guards and per-entry
/// atomics, without a table-wide lock. No lock is held across IO.
/// The slab reuses vacant slots and retains its pages until the table is dropped.
#[derive(Default)]
pub(crate) struct FileTable {
  // This owns a shared path handle, but does not contribute to FileEntry.owners.
  index: InternedPathDashMap<FileId>,
  files: Slab<FileEntry>,
}

impl FileTable {
  /// Retains the unique live identity for this path.
  ///
  /// No filesystem IO or canonicalization is performed. Path bytes are shared
  /// with InternedPath, and its cached hash is used without rehashing them.
  ///
  /// Panics if the slab's capacity, thread limit, or an entry's reference count
  /// is exhausted.
  pub(crate) fn intern(self: &Arc<Self>, path: &InternedPath) -> FileRef {
    let id = match self.index.entry(path.clone()) {
      Entry::Occupied(entry) => {
        let id = *entry.get();
        self.entry(id).retain();
        id
      }
      Entry::Vacant(entry) => {
        let id = FileId(
          self
            .files
            .insert(FileEntry {
              path: path.clone(),
              owners: AtomicU32::new(1),
            })
            .expect("filesystem table slab capacity exceeded"),
        );
        entry.insert(id);
        id
      }
    };
    FileRef {
      table: Arc::clone(self),
      id,
    }
  }

  fn entry(&self, id: FileId) -> sharded_slab::Entry<'_, FileEntry> {
    self
      .files
      .get(id.0)
      .expect("an owned file must occupy a slab slot")
  }

  fn release(&self, id: FileId) {
    let file = self.entry(id);
    if file
      .owners
      .fetch_update(Ordering::Release, Ordering::Relaxed, |owners| {
        (owners > 1).then(|| owners - 1)
      })
      .is_ok()
    {
      return;
    }

    // Keep the final owner alive until we lock the index. An intern may retain
    // it meanwhile, so decrement and check again under the path's index guard.
    let path = file.path.clone();
    let shard = self.index.determine_map(&path);
    let Entry::Occupied(entry) = self.index.entry(path) else {
      unreachable!("an owned file must be indexed");
    };
    debug_assert_eq!(*entry.get(), id);
    if file.owners.fetch_sub(1, Ordering::Release) != 1 {
      return;
    }
    // Pair with preceding owners' releases before destroying their shared entry.
    fence(Ordering::Acquire);
    // Our slab guard defers destruction until after the DashMap guard is gone.
    // Do not use take(): it would spin waiting for this guard to be dropped.
    assert!(
      self.files.remove(id.0),
      "an owned file must occupy a slab slot"
    );
    drop(entry.remove_entry());
    drop(file);

    // DashMap's public shrink_to_fit visits every shard. Its raw API lets us
    // shrink only the affected shard, with no custom hash table implementation.
    let mut index = self.index.shards()[shard].write();
    if index.len() <= index.capacity() / 4 {
      let len = index.len();
      index.shrink_to(len, |(path, _)| path.precomputed_hash());
    }
  }

  /// Collects diagnostic counts without keeping any entries alive.
  ///
  /// Counts are sampled separately, so this is not an atomic snapshot while
  /// other threads mutate the table. sharded-slab does not expose page capacity.
  pub(crate) fn statistics(&self) -> FileTableStatistics {
    FileTableStatistics {
      files: self.index.len(),
      index_capacity: self.index.capacity(),
    }
  }
}

impl fmt::Debug for FileTable {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FileTable")
      .field("statistics", &self.statistics())
      .finish()
  }
}

/// An owning reference to a file identity and its namespace.
///
/// Cloning retains the slot and table; it does not clone the FileEntry or path.
/// The final drop removes the index entry and makes the slot reusable.
pub(crate) struct FileRef {
  table: Arc<FileTable>,
  id: FileId,
}

impl FileRef {
  /// The key is valid only while an owning reference keeps this entry alive.
  pub(crate) fn id(&self) -> FileId {
    self.id
  }

  /// Returns a shared path handle, so callers never hold a lock over IO.
  pub(crate) fn path(&self) -> InternedPath {
    self.table.entry(self.id).path.clone()
  }
}

impl Clone for FileRef {
  fn clone(&self) -> Self {
    self.table.entry(self.id).retain();
    Self {
      table: Arc::clone(&self.table),
      id: self.id,
    }
  }
}

impl Drop for FileRef {
  fn drop(&mut self) {
    self.table.release(self.id);
  }
}

impl PartialEq for FileRef {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id && Arc::ptr_eq(&self.table, &other.table)
  }
}

impl Eq for FileRef {}

impl Hash for FileRef {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Arc::as_ptr(&self.table).hash(state);
    self.id.hash(state);
  }
}

impl fmt::Debug for FileRef {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FileRef")
      .field("namespace", &Arc::as_ptr(&self.table))
      .field("id", &self.id)
      .field("path", &self.path())
      .finish()
  }
}

/// Indexed live files and index allocation capacity, excluding retained slab pages.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FileTableStatistics {
  pub files: usize,
  pub index_capacity: usize,
}
