//! Common snapshot extraction, following webpack's SnapshotOptimization (MIT):
//! https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L666-L958
//!
//! Published snapshots are immutable because creation, validation and cache
//! serialization can run concurrently. Register an owned collection as a child
//! on its first capture; reuse it whole, or move a partial intersection out of
//! the new snapshot. Unlike webpack, splitting never mutates an older snapshot.
//! A shared, sharded index lets all workers reuse these immutable children.
//! Index entries are independent hints: concurrent updates may miss a sharing
//! opportunity, but do not require a transaction across paths or snapshot IDs.

use std::sync::OnceLock;

use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap, InternedPathSet};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::{SharedSnapshot, Snapshot};

const MIN_COMMON_SNAPSHOT_SIZE: usize = 3;

struct SnapshotOptimizationEntry {
  snapshot: SharedSnapshot,
  // Reuse score, including the older group when extracting an intersection.
  shared: usize,
}

type SnapshotCandidates = (
  SmallVec<[(SharedSnapshot, usize); 8]>,
  FxHashMap<SharedSnapshot, usize>,
);

#[derive(Default)]
pub(super) struct SnapshotOptimization {
  // Most snapshot strategies leave several fields unused. Allocate shards only
  // when this field first has enough entries to form a shared child.
  map: OnceLock<InternedPathDashMap<SnapshotOptimizationEntry>>,
}

impl SnapshotOptimization {
  /// webpack's complete-shared-snapshot branch runs before filesystem reads.
  /// Removing its paths also avoids cloning cached hashes.
  pub(super) fn reuse<C: SnapshotContent>(
    &self,
    new_snapshot: &mut Snapshot,
    captured_files: &mut InternedPathSet,
    get: fn(&Snapshot) -> Option<&C>,
    use_start_time: bool,
  ) {
    if captured_files.len() < MIN_COMMON_SNAPSHOT_SIZE {
      return;
    }
    let Some(map) = self.map.get() else {
      return;
    };
    let (candidates, overflow) = collect_candidates(map, captured_files.iter());
    for (snapshot, shared) in candidates.into_iter().chain(overflow) {
      if !compatible_start_time(new_snapshot, &snapshot, use_start_time) {
        continue;
      }
      let entries = get(&snapshot).expect("indexed snapshot should contain this collection");
      if entries.paths().all(|path| captured_files.contains(path)) {
        for path in entries.paths() {
          captured_files.remove(path);
        }
        store_optimization_entry(map, &snapshot, entries, shared + 1);
        new_snapshot.add_child(snapshot);
      }
    }
  }

  /// webpack's common-subset extraction, moving entries instead of cloning them.
  fn optimize<C: SnapshotContent>(
    &self,
    new_snapshot: &mut Snapshot,
    mut captured: C,
    get: fn(&Snapshot) -> Option<&C>,
    set: fn(&mut Snapshot, C),
    use_start_time: bool,
  ) {
    if captured.len() < MIN_COMMON_SNAPSHOT_SIZE {
      if captured.len() != 0 {
        set(new_snapshot, captured);
      }
      return;
    }
    let map = self.map.get_or_init(Default::default);
    let (candidates, overflow) = collect_candidates(map, captured.paths());
    for (snapshot, shared) in candidates.into_iter().chain(overflow) {
      let entries = get(&snapshot).expect("indexed snapshot should contain this collection");
      let shared_count = if captured.len() < entries.len() {
        captured
          .paths()
          .filter(|path| entries.contains(path))
          .count()
      } else {
        entries
          .paths()
          .filter(|path| captured.contains(path))
          .count()
      };
      if shared_count < MIN_COMMON_SNAPSHOT_SIZE {
        continue;
      }
      if shared_count == entries.len()
        && compatible_start_time(new_snapshot, &snapshot, use_start_time)
      {
        // Another task may have published a complete shared snapshot while
        // this task was reading the filesystem.
        for path in entries.paths() {
          captured.remove(path);
        }
        store_optimization_entry(map, &snapshot, entries, shared + 1);
        new_snapshot.add_child(snapshot);
      } else {
        // Extract a common part, or capture a group with an earlier start time.
        // The old snapshot remains immutable; subsequent captures on any
        // worker can use the new group with the merged start time.
        let common = if shared_count == captured.len() {
          // The whole new collection is common: reuse its backing allocation.
          std::mem::take(&mut captured)
        } else {
          captured.extract(entries, shared_count)
        };
        let mut common_snapshot = Snapshot {
          start_time: if use_start_time {
            match (new_snapshot.start_time, snapshot.start_time) {
              (Some(first), Some(second)) => Some(first.min(second)),
              (first, second) => first.or(second),
            }
          } else {
            None
          },
          ..Default::default()
        };
        set(&mut common_snapshot, common);
        let common_snapshot = SharedSnapshot::from(common_snapshot);
        store_optimization_entry(
          map,
          &common_snapshot,
          get(&common_snapshot).expect("common snapshot should contain this collection"),
          shared + 1,
        );
        new_snapshot.add_child(common_snapshot);
      }
    }
    if captured.len() >= MIN_COMMON_SNAPSHOT_SIZE {
      // Publish the remaining collection once. Later captures can reference it
      // without rewriting a snapshot another task may already be reading.
      let mut snapshot = Snapshot {
        start_time: if use_start_time {
          new_snapshot.start_time
        } else {
          None
        },
        ..Default::default()
      };
      set(&mut snapshot, captured);
      let snapshot = SharedSnapshot::from(snapshot);
      store_optimization_entry(
        map,
        &snapshot,
        get(&snapshot).expect("snapshot should contain this collection"),
        1,
      );
      new_snapshot.add_child(snapshot);
    } else if captured.len() != 0 {
      set(new_snapshot, captured);
    }
  }
}

fn collect_candidates<'a>(
  map: &InternedPathDashMap<SnapshotOptimizationEntry>,
  paths: impl Iterator<Item = &'a InternedPath>,
) -> SnapshotCandidates {
  // Keep common cases on the stack. Hash only the overflow so a large number
  // of overlapping groups does not turn candidate deduplication quadratic.
  let mut candidates: SmallVec<[(SharedSnapshot, usize); 8]> = SmallVec::new();
  let mut overflow = FxHashMap::default();
  for path in paths {
    let Some(entry) = map.get(path) else {
      continue;
    };
    if candidates
      .iter()
      .any(|(snapshot, _)| *snapshot == entry.snapshot)
    {
      continue;
    }
    if candidates.len() < candidates.inline_size() {
      candidates.push((entry.snapshot.clone(), entry.shared));
    } else if !overflow.contains_key(&entry.snapshot) {
      overflow.insert(entry.snapshot.clone(), entry.shared);
    }
  }
  // All shard guards are released before checking or extracting intersections.
  (candidates, overflow)
}

fn compatible_start_time(new: &Snapshot, old: &Snapshot, use_start_time: bool) -> bool {
  match (use_start_time, new.start_time, old.start_time) {
    (true, Some(new), Some(old)) => old <= new,
    (true, Some(_), None) => false,
    _ => true,
  }
}

fn store_optimization_entry<C: SnapshotContent>(
  map: &InternedPathDashMap<SnapshotOptimizationEntry>,
  snapshot: &SharedSnapshot,
  entries: &C,
  shared: usize,
) {
  for path in entries.paths() {
    // Avoid cloning an already indexed path. On a miss, entry() also handles
    // another worker inserting the same path before we acquire the write lock.
    let mut entry = map.get_mut(path).unwrap_or_else(|| {
      map
        .entry(path.clone())
        .or_insert_with(|| SnapshotOptimizationEntry {
          snapshot: snapshot.clone(),
          shared,
        })
    });
    if entry.shared < shared {
      if entry.snapshot != *snapshot {
        entry.snapshot = snapshot.clone();
      }
      entry.shared = shared;
    }
  }
}

/// The same algorithm handles webpack's Map fields and managed-path Sets.
pub(super) trait SnapshotContent: Default {
  fn len(&self) -> usize;
  fn paths(&self) -> impl Iterator<Item = &InternedPath>;
  fn contains(&self, path: &InternedPath) -> bool;
  fn remove(&mut self, path: &InternedPath);
  fn extract(&mut self, common: &Self, count: usize) -> Self;
}

impl<T> SnapshotContent for InternedPathMap<T> {
  fn len(&self) -> usize {
    self.len()
  }

  fn paths(&self) -> impl Iterator<Item = &InternedPath> {
    self.keys()
  }

  fn contains(&self, path: &InternedPath) -> bool {
    self.contains_key(path)
  }

  fn remove(&mut self, path: &InternedPath) {
    self.remove(path);
  }

  fn extract(&mut self, common: &Self, count: usize) -> Self {
    let mut result = Self::with_capacity_and_hasher(count, Default::default());
    if self.len() < common.len() {
      result.extend(self.extract_if(|path, _| common.contains_key(path)));
    } else {
      for path in common.keys() {
        if let Some((path, value)) = self.remove_entry(path) {
          result.insert(path, value);
        }
      }
    }
    result
  }
}

impl SnapshotContent for InternedPathSet {
  fn len(&self) -> usize {
    self.len()
  }

  fn paths(&self) -> impl Iterator<Item = &InternedPath> {
    self.iter()
  }

  fn contains(&self, path: &InternedPath) -> bool {
    self.contains(path)
  }

  fn remove(&mut self, path: &InternedPath) {
    self.remove(path);
  }

  fn extract(&mut self, common: &Self, count: usize) -> Self {
    let mut result = Self::with_capacity_and_hasher(count, Default::default());
    if self.len() < common.len() {
      result.extend(self.extract_if(|path| common.contains(path)));
    } else {
      for path in common {
        if let Some(path) = self.take(path) {
          result.insert(path);
        }
      }
    }
    result
  }
}

#[derive(Default)]
pub(super) struct SnapshotOptimizations {
  pub(super) file_timestamps: SnapshotOptimization,
  pub(super) file_hashes: SnapshotOptimization,
  pub(super) file_timestamp_hashes: SnapshotOptimization,
  pub(super) context_timestamps: SnapshotOptimization,
  pub(super) context_hashes: SnapshotOptimization,
  pub(super) context_timestamp_hashes: SnapshotOptimization,
  pub(super) missing_existence: SnapshotOptimization,
  managed_item_info: SnapshotOptimization,
  managed_files: SnapshotOptimization,
  managed_contexts: SnapshotOptimization,
  managed_missing: SnapshotOptimization,
}

impl SnapshotOptimizations {
  pub(super) fn optimize(&self, snapshot: &mut Snapshot) {
    macro_rules! optimize {
      ($field:ident, $use_start_time:literal) => {
        if let Some(captured) = snapshot.$field.take() {
          self.$field.optimize(
            snapshot,
            captured,
            |snapshot| snapshot.$field.as_ref(),
            |snapshot, value| snapshot.$field = Some(value),
            $use_start_time,
          );
        }
      };
    }

    optimize!(file_timestamps, true);
    optimize!(file_hashes, false);
    optimize!(file_timestamp_hashes, true);
    optimize!(context_timestamps, true);
    optimize!(context_hashes, false);
    optimize!(context_timestamp_hashes, true);
    optimize!(missing_existence, false);
    optimize!(managed_item_info, false);
    optimize!(managed_files, false);
    optimize!(managed_contexts, false);
    optimize!(managed_missing, false);
  }
}
