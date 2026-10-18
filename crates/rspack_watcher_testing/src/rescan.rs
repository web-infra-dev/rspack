use std::time::{Duration, SystemTime};

use rspack_paths::InternedPath;

use crate::{
  FsEventKind,
  dirwatch::{ScanReason, WatcherManager},
  state::Origin,
};

#[tokio::test]
async fn non_initial_scan_reports_changes_including_registered_files() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let changed = InternedPath::from(root.join("changed.js"));
  let removed = InternedPath::from(root.join("removed.js"));
  let stable = InternedPath::from(root.join("stable.js"));
  for path in [&changed, &removed, &stable] {
    std::fs::write(path, "old").expect("file");
  }
  let manager = WatcherManager::default();
  manager
    .update(
      (vec![changed.clone()], vec![]),
      (vec![root.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  let time = SystemTime::UNIX_EPOCH + Duration::from_secs(123);
  std::fs::File::options()
    .write(true)
    .open(&changed)
    .expect("open")
    .set_modified(time)
    .expect("mtime");
  std::fs::remove_file(&removed).expect("remove");
  let added = InternedPath::from(root.join("added.js"));
  std::fs::write(&added, "new").expect("add");
  let changes = manager
    .scan(vec![root.clone()], ScanReason::Discovered)
    .await;
  for path in [&changed, &removed, &added] {
    assert!(
      changes.contains(path),
      "missing change {path:?}: {changes:?}"
    );
  }
  assert!(!changes.contains(&stable));
  assert!(
    manager
      .scan(vec![root], ScanReason::Discovered)
      .await
      .is_empty(),
    "unchanged rescan must be silent"
  );
}

#[tokio::test]
async fn discovered_directory_notifies_registered_file_without_appeared_flag() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let child = InternedPath::from(root.join("child"));
  let file = InternedPath::from(child.join("entry.js"));
  let manager = WatcherManager::default();
  manager
    .update(
      (vec![file.clone()], vec![]),
      (vec![root], vec![]),
      Default::default(),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  std::fs::create_dir(&child).expect("child");
  std::fs::write(&file, "new").expect("file");
  let events = manager
    .process(
      &child,
      FsEventKind::Change,
      Origin::Disk { appeared: false },
    )
    .await
    .expect("events");
  assert!(
    events
      .iter()
      .any(|(path, kind)| path == &file && *kind != FsEventKind::Remove),
    "scan must notify registered child: {events:?}"
  );
  assert!(
    manager
      .process(&file, FsEventKind::Change, Origin::Disk { appeared: false })
      .await
      .is_none(),
    "the scan notification must advance the file baseline"
  );
}

async fn vanished_subtree(replace_with_file: bool, scan_root: bool, follow_symlinks: bool) {
  let temp = tempfile::tempdir().expect("directory");
  let archive = tempfile::tempdir().expect("archive");
  let root = InternedPath::from(temp.path());
  let child = InternedPath::from(root.join("child"));
  let context = InternedPath::from(child.join("context"));
  let file = InternedPath::from(child.join("entry.js"));
  let found = InternedPath::from(child.join("found.js"));
  let absent = InternedPath::from(child.join("absent.js"));
  std::fs::create_dir_all(&context).expect("context");
  for path in [&file, &found] {
    std::fs::write(path, "old").expect("file");
  }
  let manager = WatcherManager::new(crate::FsWatcherIgnored::default(), follow_symlinks);
  manager
    .update(
      (vec![file.clone()], vec![]),
      (vec![root.clone(), context.clone()], vec![]),
      (vec![found.clone(), absent.clone()], vec![]),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  manager.reset();
  std::fs::rename(&child, archive.path().join("child")).expect("move subtree");
  if replace_with_file {
    std::fs::write(&child, "file").expect("replace subtree");
  }
  let changes = manager
    .scan(
      vec![if scan_root { root } else { child }],
      ScanReason::Discovered,
    )
    .await;
  for path in [&file, &found, &context] {
    assert!(
      changes.contains(path),
      "lost descendant {path:?}: {changes:?}"
    );
  }
  assert!(
    !changes.contains(&absent),
    "never-present missing must stay silent"
  );
  let mut events = Vec::new();
  for path in changes {
    events.extend(
      manager
        .process(&path, FsEventKind::Change, Origin::Scan)
        .await
        .unwrap_or_default(),
    );
  }
  for path in [&file, &found, &context] {
    assert!(
      events.contains(&(path.clone(), FsEventKind::Remove)),
      "scan changes must reach subscribers: {events:?}"
    );
  }
  assert!(!events.iter().any(|(path, _)| path == &absent));
  assert!(manager.entry(&file).is_none());
  assert!(manager.entry(&found).is_none());
  assert!(
    manager
      .watchers
      .get(&context)
      .expect("context watcher")
      .absent
  );
}

#[tokio::test]
async fn scanning_parent_reports_removed_subtree_dependencies() {
  vanished_subtree(false, true, false).await;
}

#[tokio::test]
async fn scanning_missing_root_reports_registered_descendants() {
  vanished_subtree(false, false, false).await;
}

#[tokio::test]
async fn canonicalization_failure_reports_registered_descendants() {
  vanished_subtree(false, false, true).await;
}

#[tokio::test]
async fn scanning_directory_replaced_by_file_reports_descendants() {
  vanished_subtree(true, true, false).await;
}

#[tokio::test]
async fn repeated_scans_of_a_removed_context_are_silent_until_recovery() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path().join("context"));
  let nested = InternedPath::from(root.join("nested"));
  let file = InternedPath::from(nested.join("entry.js"));
  std::fs::create_dir_all(&nested).expect("nested");
  std::fs::write(&file, "entry").expect("file");
  let manager = WatcherManager::default();
  manager
    .update(
      (vec![file.clone()], vec![]),
      (vec![root.clone(), nested.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  manager.reset();
  std::fs::remove_dir_all(&root).expect("remove context");
  let removed = manager
    .scan(vec![root.clone()], ScanReason::Discovered)
    .await;
  for path in [&root, &nested, &file] {
    assert!(
      removed.contains(path),
      "missing removal {path:?}: {removed:?}"
    );
  }
  assert!(
    manager
      .scan(vec![root.clone()], ScanReason::Discovered)
      .await
      .is_empty(),
    "an absent context must not repeat removals"
  );
  std::fs::create_dir_all(&nested).expect("recover nested");
  std::fs::write(&file, "restored").expect("recover file");
  let restored = manager
    .scan(vec![root.clone()], ScanReason::Discovered)
    .await;
  assert!(restored.contains(&file));
  assert!(restored.contains(&root), "recovered root must notify");
  assert!(
    manager
      .scan(vec![root.clone()], ScanReason::Discovered)
      .await
      .is_empty(),
    "unchanged recovered context must be silent"
  );
  std::fs::remove_dir_all(&root).expect("remove again");
  let removed = manager.scan(vec![root], ScanReason::Discovered).await;
  assert!(
    removed.contains(&file),
    "the next deletion must still notify"
  );
}
