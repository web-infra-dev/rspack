use std::{sync::Arc, time::SystemTime};

use rspack_paths::InternedPath;

use crate::{FsEventKind, FsWatcherIgnored, dirwatch::WatcherManager, state::Origin};

async fn ancestor_event(replace_with_file: bool, recreate_directory: bool) {
  let temp = tempfile::tempdir().expect("temp directory");
  let archive = tempfile::tempdir().expect("archive");
  let parent = InternedPath::from(temp.path().join("parent"));
  let context = InternedPath::from(parent.join("context"));
  std::fs::create_dir_all(&context).expect("context");
  let file = InternedPath::from(parent.join("entry.js"));
  let found = InternedPath::from(parent.join("found.js"));
  let absent = InternedPath::from(parent.join("absent.js"));
  for path in [&file, &found] {
    std::fs::write(path, "old").expect("old file");
  }
  let manager = Arc::new(WatcherManager::new(FsWatcherIgnored::default(), false));
  manager
    .update(
      (vec![file.clone()], vec![]),
      (vec![context.clone()], vec![]),
      (vec![found.clone(), absent.clone()], vec![]),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  manager.scan_registered(SystemTime::UNIX_EPOCH).await;
  std::fs::rename(&parent, archive.path().join("old")).expect("move old parent");
  if replace_with_file {
    std::fs::write(&parent, "file").expect("replace parent with file");
  }
  if recreate_directory {
    std::fs::create_dir_all(&context).expect("restore context");
    std::fs::write(&file, "new").expect("restore file");
  }
  let kind = if recreate_directory {
    FsEventKind::Remove
  } else {
    FsEventKind::Change
  };
  let events = manager
    .process(&parent, kind, Origin::Disk { appeared: false })
    .await
    .expect("ancestor event");
  let expected = if recreate_directory {
    FsEventKind::Change
  } else {
    FsEventKind::Remove
  };
  assert!(
    events.contains(&(file.clone(), expected)),
    "registered file must receive ancestor change: {events:?}"
  );
  assert!(
    events.contains(&(context.clone(), expected)),
    "registered context must receive ancestor change"
  );
  assert!(
    events.contains(&(found.clone(), FsEventKind::Remove)),
    "previously found missing dependency was removed"
  );
  assert!(
    !events.iter().any(|(path, _)| path == &absent),
    "never-present missing dependency did not change"
  );
  assert_eq!(
    manager
      .watchers
      .get(&context)
      .expect("context watcher")
      .absent,
    !recreate_directory
  );
  assert!(manager.entry(&found).is_none());
  if recreate_directory {
    assert!(manager.entry(&file).expect("restored entry").time.is_some());
  } else {
    assert!(manager.entry(&file).is_none());
  }
}

#[tokio::test]
async fn removed_ancestor_notifies_registered_descendants() {
  ancestor_event(false, false).await;
}
#[tokio::test]
async fn ancestor_replaced_by_file_notifies_registered_descendants() {
  ancestor_event(true, false).await;
}
#[tokio::test]
async fn stale_ancestor_removal_reports_each_descendants_current_state() {
  ancestor_event(false, true).await;
}

async fn appeared_replacement(watch_context: bool) {
  let temp = tempfile::tempdir().expect("directory");
  let archive = tempfile::tempdir().expect("archive");
  let parent = InternedPath::from(temp.path().join("parent"));
  let context = InternedPath::from(parent.join("context"));
  let removed = InternedPath::from(parent.join("removed.js"));
  let retained = InternedPath::from(parent.join("retained.js"));
  let found = InternedPath::from(parent.join("found.js"));
  let added = InternedPath::from(parent.join("added.js"));
  let absent = InternedPath::from(parent.join("absent.js"));
  std::fs::create_dir_all(&context).expect("context");
  for path in [&removed, &retained, &found] {
    std::fs::write(path, "old").expect("old file");
  }
  let old_mtime = std::fs::metadata(&retained)
    .expect("metadata")
    .modified()
    .expect("mtime");
  let manager = Arc::new(WatcherManager::new(FsWatcherIgnored::default(), false));
  let mut contexts = vec![context.clone()];
  if watch_context {
    contexts.push(parent.clone());
  }
  manager
    .update(
      (vec![removed.clone(), retained.clone()], vec![]),
      (contexts, vec![]),
      (vec![found.clone(), added.clone(), absent.clone()], vec![]),
    )
    .await
    .expect("register");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  manager.scan_registered(SystemTime::UNIX_EPOCH).await;
  manager.reset();
  std::fs::rename(&parent, archive.path().join("old")).expect("archive old");
  std::fs::create_dir(&parent).expect("replacement");
  std::fs::write(&retained, "new").expect("retained replacement");
  std::fs::File::options()
    .write(true)
    .open(&retained)
    .expect("open")
    .set_modified(old_mtime)
    .expect("preserve mtime");
  std::fs::write(&added, "new").expect("new missing dependency");
  let events = manager
    .process(
      &parent,
      FsEventKind::Create,
      Origin::Disk { appeared: true },
    )
    .await
    .expect("events");
  for path in [&removed, &found, &context] {
    assert!(
      events.contains(&(path.clone(), FsEventKind::Remove)),
      "removed descendant {path:?}: {events:?}"
    );
  }
  assert!(
    events.contains(&(retained.clone(), FsEventKind::Change)),
    "same-mtime replacement must notify: {events:?}"
  );
  assert!(events.contains(&(added, FsEventKind::Create)));
  assert!(!events.iter().any(|(path, _)| path == &absent));
  assert!(manager.entry(&removed).is_none());
  assert!(manager.entry(&found).is_none());
  assert!(
    manager
      .watchers
      .get(&context)
      .expect("context watcher")
      .absent
  );
  assert!(
    manager
      .process(
        &retained,
        FsEventKind::Change,
        Origin::Disk { appeared: false }
      )
      .await
      .is_none(),
    "replacement must update baseline"
  );
}

#[tokio::test]
async fn appeared_context_replacement_reconciles_registered_descendants() {
  appeared_replacement(true).await;
}

#[tokio::test]
async fn appeared_ancestor_replacement_reconciles_registered_descendants() {
  appeared_replacement(false).await;
}
