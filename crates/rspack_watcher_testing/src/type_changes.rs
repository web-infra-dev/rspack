use std::time::SystemTime;

use rspack_paths::InternedPath;

use crate::{FsEventKind, FsWatcherIgnored, dirwatch::WatcherManager, state::Origin};

async fn file_to_directory(disk_event: bool) {
  let temp = tempfile::tempdir().expect("temp directory");
  let root = InternedPath::from(temp.path());
  let path = InternedPath::from(root.join("swap"));
  std::fs::write(&path, "old").expect("old file");
  let manager = WatcherManager::new(FsWatcherIgnored::default(), false);
  manager
    .update(
      (vec![path.clone()], vec![]),
      (vec![root.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  assert!(manager.entry(&path).is_some());
  std::fs::remove_file(&path).expect("remove test file");
  std::fs::create_dir(&path).expect("replacement directory");
  let child = InternedPath::from(path.join("new.js"));
  std::fs::write(&child, "new").expect("new child");
  if disk_event {
    manager
      .process(&path, FsEventKind::Change, Origin::Disk { appeared: true })
      .await
      .expect("change event");
  } else {
    manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  }
  assert!(
    manager
      .associated_events(&path, FsEventKind::Change)
      .contains(&(path.clone(), FsEventKind::Change)),
    "registered file still receives replacement changes"
  );
  assert!(
    manager.entry(&path).is_none(),
    "directory must not retain old file time or baseline"
  );
  assert!(
    manager
      .watchers
      .get(&root)
      .expect("root")
      .directories
      .contains(&path)
  );
  assert!(manager.entry(&child).is_some());
  assert!(
    manager.registry.files.all.contains(&path),
    "type replacement must not unregister the dependency"
  );
}

async fn directory_to_file(disk_event: bool) {
  let temp = tempfile::tempdir().expect("temp directory");
  let archive = tempfile::tempdir().expect("archive");
  let root = InternedPath::from(temp.path());
  let path = InternedPath::from(root.join("swap"));
  let deep = InternedPath::from(path.join("deep"));
  std::fs::create_dir_all(&deep).expect("old subtree");
  let child = InternedPath::from(deep.join("old.js"));
  std::fs::write(&child, "old").expect("old child");
  let manager = WatcherManager::new(FsWatcherIgnored::default(), false);
  manager
    .update(
      (vec![child.clone()], vec![]),
      (vec![root.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  std::fs::rename(&path, archive.path().join("old")).expect("move old subtree away");
  std::fs::write(&path, "new").expect("replacement file");
  if disk_event {
    manager
      .process(&path, FsEventKind::Change, Origin::Disk { appeared: false })
      .await
      .expect("change event");
  } else {
    manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  }
  assert!(
    !manager
      .watchers
      .get(&root)
      .expect("root")
      .directories
      .contains(&path),
    "file must not remain a directory entry"
  );
  assert!(manager.entry(&path).expect("new file").regular_file);
  assert!(
    manager.entry(&child).is_none(),
    "old subtree times must be discarded"
  );
  assert!(
    !manager.watchers.contains_key(&path),
    "unreferenced nested watcher must be removed"
  );
  let retained = manager
    .watchers
    .get(&deep)
    .expect("registered dependency retains its watcher");
  assert_eq!(retained.registered_files, 1);
  assert!(retained.files.is_empty());
  assert!(retained.initial_scan_finished.is_none());
  assert!(!retained.nested_watching);
  assert!(manager.registry.files.all.contains(&child));
}

#[tokio::test]
async fn scan_replaces_file_with_directory() {
  file_to_directory(false).await;
}
#[tokio::test]
async fn event_replaces_file_with_directory() {
  file_to_directory(true).await;
}
#[tokio::test]
async fn scan_replaces_directory_with_file() {
  directory_to_file(false).await;
}
#[tokio::test]
async fn event_replaces_directory_with_file() {
  directory_to_file(true).await;
}

#[tokio::test]
async fn a_registered_context_replaced_by_a_file_is_absent_and_notified() {
  let temp = tempfile::tempdir().expect("temp directory");
  let path = InternedPath::from(temp.path().join("context"));
  std::fs::create_dir(&path).expect("context directory");
  let manager = WatcherManager::new(FsWatcherIgnored::default(), false);
  manager
    .update(
      Default::default(),
      (vec![path.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register context");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  std::fs::remove_dir(&path).expect("remove empty test directory");
  std::fs::write(&path, "replacement").expect("replacement file");
  let events = manager
    .process(&path, FsEventKind::Change, Origin::Disk { appeared: true })
    .await
    .expect("replacement event");
  assert!(
    events.contains(&(path.clone(), FsEventKind::Change)),
    "context subscriber must be notified even when target is now a file"
  );
  assert!(manager.watchers.get(&path).expect("context watcher").absent);
  let (_, directories) = manager.collect_time_info_entries();
  assert!(
    directories
      .iter()
      .any(|(name, time)| name == path.to_string_lossy().as_ref()
        && matches!(time, crate::TimeInfoEntry::Null)),
    "a file cannot provide directory time info"
  );
  std::fs::remove_file(&path).expect("remove replacement test file");
  std::fs::create_dir(&path).expect("restore context directory");
  let child = InternedPath::from(path.join("restored.js"));
  std::fs::write(&child, "restored").expect("restored child");
  manager
    .process(&path, FsEventKind::Change, Origin::Disk { appeared: true })
    .await
    .expect("recovery event");
  assert!(!manager.watchers.get(&path).expect("context watcher").absent);
  assert!(manager.entry(&child).is_some());
}
