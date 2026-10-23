use std::{
  collections::HashSet,
  mem::size_of,
  sync::{Arc, Barrier},
  thread,
};

use rspack_paths::InternedPath;

// Exercise the private implementation without exporting a new rspack_core API.
#[path = "../../rspack_core/src/new_cache/fs_info/mod.rs"]
mod fs_info;

use fs_info::{FileId, FileRef, FileTable, FileTableStatistics};

fn path(index: usize) -> InternedPath {
  InternedPath::from(format!("/project/src/file-{index}.js").as_str())
}

#[test]
fn test_file_id_preserves_the_slab_key_and_owners_are_thread_safe() {
  fn assert_send_sync<T: Send + Sync>() {}

  assert_eq!(size_of::<FileId>(), size_of::<usize>());
  assert_send_sync::<FileTable>();
  assert_send_sync::<FileRef>();
}

#[test]
fn test_intern_shares_one_live_entry_until_the_last_owner_drops() {
  let table = Arc::new(FileTable::default());
  let path = path(0);
  let first = table.intern(&path);
  let second = table.intern(&InternedPath::from(path.as_path()));
  let cloned = first.clone();

  assert_eq!(first, second);
  assert_eq!(first.id(), cloned.id());
  assert_eq!(table.statistics().files, 1);
  drop(first);
  drop(second);
  assert_eq!(cloned.path(), path);
  assert_eq!(table.statistics().files, 1);
  drop(cloned);
  assert_eq!(table.statistics(), FileTableStatistics::default());
}

#[test]
fn test_namespaces_keep_identical_paths_independent() {
  let first_table = Arc::new(FileTable::default());
  let second_table = Arc::new(FileTable::default());
  let path = path(0);
  let first = first_table.intern(&path);
  let second = second_table.intern(&path);

  // Slab keys are local to each table, even when their bits coincide.
  assert_eq!(first.id(), second.id());
  assert_ne!(first, second);
  drop(first);
  assert_eq!(first_table.statistics(), FileTableStatistics::default());
  assert_eq!(second_table.statistics().files, 1);
  assert_eq!(second.path(), path);
}

#[test]
fn test_file_owner_keeps_its_table_alive_without_a_cycle() {
  let table = Arc::new(FileTable::default());
  let weak = Arc::downgrade(&table);
  let file = table.intern(&path(0));
  drop(table);

  assert!(weak.upgrade().is_some());
  assert_eq!(file.path(), path(0));
  drop(file);
  assert!(weak.upgrade().is_none());
}

#[test]
fn test_owned_path_survives_entry_release_and_slot_reuse() {
  let table = Arc::new(FileTable::default());
  let file = table.intern(&path(0));
  let id = file.id();
  let owned_path = file.path();
  drop(file);
  assert_eq!(table.statistics(), FileTableStatistics::default());

  let replacement = table.intern(&path(0));
  // Keep the complete generational key when a slot is reused.
  assert_ne!(id, replacement.id());
  assert_eq!(owned_path, replacement.path());
  drop(replacement);
  assert_eq!(table.statistics(), FileTableStatistics::default());
  assert_eq!(owned_path, path(0));
}

#[test]
fn test_releasing_files_preserves_survivors_and_shrinks_the_index() {
  let table = Arc::new(FileTable::default());
  let mut files: Vec<_> = (0..16_384)
    .map(|index| table.intern(&path(index)))
    .collect();
  let ids: HashSet<_> = files.iter().map(FileRef::id).collect();
  assert_eq!(ids.len(), files.len());
  let survivor = files.pop().expect("the last file exists");
  let id = survivor.id();
  let peak = table.statistics();
  assert_eq!(peak.files, 16_384);
  drop(files);

  let remaining = table.statistics();
  assert_eq!(remaining.files, 1);
  assert!(remaining.index_capacity < peak.index_capacity);
  assert_eq!(survivor.id(), id);
  assert_eq!(survivor.path(), path(16_383));
  assert_eq!(table.intern(&path(16_383)), survivor);
  drop(survivor);
  assert_eq!(table.statistics(), FileTableStatistics::default());
}

#[test]
fn test_churn_releases_entries_and_index_capacity() {
  let table = Arc::new(FileTable::default());
  for round in 0..8 {
    let files: Vec<_> = (0..2048)
      .map(|index| table.intern(&path(round * 2048 + index)))
      .collect();
    assert_eq!(table.statistics().files, files.len());
    drop(files);
    assert_eq!(table.statistics(), FileTableStatistics::default());
  }
}

#[test]
fn test_churn_preserves_live_identity_while_other_paths_change() {
  let table = Arc::new(FileTable::default());
  let paths: Vec<_> = (0..256 + 4 * 255).map(path).collect();
  let mut files: Vec<_> = paths[..256].iter().map(|path| table.intern(path)).collect();
  let survivor = files.pop().expect("the last file exists");
  let id = survivor.id();
  drop(files);

  for paths in paths[256..].chunks(255) {
    let files: Vec<_> = paths.iter().map(|path| table.intern(path)).collect();
    assert_eq!(table.statistics().files, 256);
    assert_eq!(survivor.id(), id);
    drop(files);
    assert_eq!(table.statistics().files, 1);
  }
  assert_eq!(survivor.path(), paths[255]);
  drop(survivor);
  assert_eq!(table.statistics(), FileTableStatistics::default());
}

#[test]
fn test_concurrent_interns_share_one_live_identity() {
  let table = Arc::new(FileTable::default());
  let barrier = Barrier::new(8);
  let files = thread::scope(|scope| {
    let threads: Vec<_> = (0..8)
      .map(|_| {
        scope.spawn(|| {
          barrier.wait();
          table.intern(&path(0))
        })
      })
      .collect();
    threads
      .into_iter()
      .map(|thread| thread.join().expect("intern must not panic"))
      .collect::<Vec<_>>()
  });

  assert!(files.iter().all(|file| *file == files[0]));
  assert_eq!(table.statistics().files, 1);
  drop(files);
  assert_eq!(table.statistics(), FileTableStatistics::default());
}

#[test]
fn test_many_concurrent_clones_share_one_live_entry() {
  let table = Arc::new(FileTable::default());
  let file = table.intern(&path(0));
  let barrier = Barrier::new(8);
  let mut batches = thread::scope(|scope| {
    let threads: Vec<_> = (0..8)
      .map(|_| {
        scope.spawn(|| {
          barrier.wait();
          (0..512).map(|_| file.clone()).collect::<Vec<_>>()
        })
      })
      .collect();
    threads
      .into_iter()
      .map(|thread| thread.join().expect("clone must not panic"))
      .collect::<Vec<_>>()
  });
  assert!(batches.iter().flatten().all(|cloned| *cloned == file));
  assert_eq!(table.statistics().files, 1);
  let survivor = batches[0].pop().expect("the first batch is nonempty");
  drop(file);
  thread::scope(|scope| {
    for batch in batches {
      scope.spawn(move || drop(batch));
    }
  });
  assert_eq!(table.statistics().files, 1);
  assert_eq!(survivor.path(), path(0));
  drop(survivor);
  assert_eq!(table.statistics(), FileTableStatistics::default());
}

#[test]
fn test_concurrent_last_drop_and_reintern_preserve_live_owners() {
  let table = Arc::new(FileTable::default());
  let paths: Vec<_> = (0..32).map(path).collect();
  let barrier = Barrier::new(8);
  thread::scope(|scope| {
    for _ in 0..8 {
      scope.spawn(|| {
        barrier.wait();
        for iteration in 0..4096 {
          let path = &paths[iteration % paths.len()];
          let first = table.intern(path);
          if iteration % 4 == 0 {
            thread::yield_now();
          }
          let second = table.intern(path);
          assert_eq!(first, second);
          let cloned = first.clone();
          drop(first);
          drop(second);
          assert_eq!(cloned.path(), *path);
          // This may race with another thread creating the same identity again.
          drop(cloned);
        }
      });
    }
  });
  assert_eq!(table.statistics(), FileTableStatistics::default());
}
