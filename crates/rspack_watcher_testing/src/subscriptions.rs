use std::{
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;

use crate::{FsEventKind, FsWatcherIgnored, dirwatch::WatcherManager, state::Origin};

#[tokio::test]
async fn attaching_an_existing_subdirectory_reuses_its_observed_state() {
  let temp = tempfile::tempdir().expect("temp directory");
  let root = InternedPath::from(temp.path());
  let inner = InternedPath::from(root.join("inner"));
  std::fs::create_dir(&inner).expect("inner directory");
  let path = InternedPath::from(inner.join("entry.js"));
  let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
  let file = std::fs::File::create(&path).expect("file");
  file.set_modified(old).expect("old file time");
  let visits = Arc::new(AtomicUsize::new(0));
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let visits = Arc::clone(&visits);
    let path = path.clone();
    move |value| {
      if std::path::Path::new(&value) == path.as_ref() as &std::path::Path {
        visits.fetch_add(1, Ordering::SeqCst);
      }
      Box::pin(async { false })
    }
  }));
  let manager = Arc::new(WatcherManager::new(ignored, false));
  manager
    .update(Default::default(), (vec![root], vec![]), Default::default())
    .await
    .expect("register root");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  let start = SystemTime::now();
  // A rename replacement can preserve mtime while changing the file contents.
  manager
    .process(&path, FsEventKind::Remove, Origin::Disk { appeared: true })
    .await
    .expect("replacement event");
  let mut options = std::fs::File::options();
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    options.write(true).custom_flags(0x0200_0000);
  }
  #[cfg(not(windows))]
  options.read(true);
  options
    .open(&inner)
    .expect("directory handle")
    .set_modified(old)
    .expect("old directory time");
  let before = visits.load(Ordering::SeqCst);
  manager.reset();
  manager
    .update(
      Default::default(),
      (vec![inner.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("attach inner context");
  let scanned = manager.scan_contexts(start).await;
  let batches = manager.scan_registered(start).await;
  assert!(
    scanned.contains(&inner)
      || batches
        .iter()
        .flatten()
        .any(|event| event.path == inner && event.kind == FsEventKind::Change),
    "attach reports cached observation despite old directory mtime"
  );
  assert_eq!(
    visits.load(Ordering::SeqCst),
    before,
    "existing subtree must not run ignored predicates again"
  );
  let future = SystemTime::now() + Duration::from_secs(60);
  assert!(manager.scan_contexts(future).await.is_empty());
  assert!(manager.scan_registered(future).await.is_empty());
}

#[tokio::test]
async fn an_ignored_event_requires_a_fresh_scan_on_attach() {
  use std::sync::atomic::AtomicBool;
  let temp = tempfile::tempdir().expect("temp directory");
  let root = InternedPath::from(temp.path());
  let inner = InternedPath::from(root.join("inner"));
  std::fs::create_dir(&inner).expect("inner directory");
  let path = InternedPath::from(inner.join("entry.js"));
  std::fs::write(&path, "entry").expect("file");
  let hide = Arc::new(AtomicBool::new(false));
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let hide = Arc::clone(&hide);
    move |path| {
      let ignored = hide.load(Ordering::SeqCst) && path.ends_with("entry.js");
      Box::pin(async move { ignored })
    }
  }));
  let manager = WatcherManager::new(ignored, false);
  manager
    .update(Default::default(), (vec![root], vec![]), Default::default())
    .await
    .expect("register root");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  let changed = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
  std::fs::File::options()
    .write(true)
    .open(&path)
    .expect("open file")
    .set_modified(changed)
    .expect("change mtime");
  hide.store(true, Ordering::SeqCst);
  assert!(
    manager
      .process(&path, FsEventKind::Change, Origin::Disk { appeared: false })
      .await
      .is_none()
  );
  hide.store(false, Ordering::SeqCst);
  manager.reset();
  manager
    .update(
      Default::default(),
      (vec![inner], vec![]),
      Default::default(),
    )
    .await
    .expect("attach inner");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  assert_eq!(
    manager
      .entry(&path)
      .expect("entry")
      .time
      .expect("time")
      .mtime(),
    changed
  );
}

#[tokio::test]
async fn file_and_missing_subscriptions_report_cached_observations() {
  for missing in [false, true] {
    let temp = tempfile::tempdir().expect("temp directory");
    let root = InternedPath::from(temp.path());
    let path = InternedPath::from(root.join("entry.js"));
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
    let file = std::fs::File::create(&path).expect("file");
    file.set_modified(old).expect("old mtime");
    let manager = Arc::new(WatcherManager::new(FsWatcherIgnored::default(), false));
    manager
      .update(Default::default(), (vec![root], vec![]), Default::default())
      .await
      .expect("register context");
    manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
    let start = SystemTime::now();
    manager
      .process(&path, FsEventKind::Remove, Origin::Disk { appeared: true })
      .await
      .expect("replacement event");
    let observed = manager.entry(&path).expect("entry").time;
    manager.reset();
    let added = (vec![path.clone()], vec![]);
    let (files, missing_paths) = if missing {
      (Default::default(), added)
    } else {
      (added, Default::default())
    };
    manager
      .update(files, Default::default(), missing_paths)
      .await
      .expect("attach path");
    assert!(
      manager
        .scan_registered(start + Duration::from_secs(60))
        .await
        .is_empty()
    );
    assert!(
      manager.entry(&path).expect("entry").baseline.is_none(),
      "unreported observation must not advance baseline"
    );
    let batches = manager.scan_registered(start).await;
    let kind = if missing {
      FsEventKind::Create
    } else {
      FsEventKind::Change
    };
    assert!(
      batches
        .iter()
        .flatten()
        .any(|event| event.path == path && event.kind == kind),
      "cached observation must report despite old mtime, missing={missing}"
    );
    let entry = manager.entry(&path).expect("entry");
    assert_eq!(entry.time, observed);
    assert_eq!(entry.baseline, Some(old));
  }
}

#[cfg(unix)]
#[tokio::test]
async fn a_cached_symlink_does_not_replace_its_targets_backfill() {
  let temp = tempfile::tempdir().expect("temp directory");
  let root = InternedPath::from(temp.path());
  let target = root.join("target.js");
  let path = InternedPath::from(root.join("link.js"));
  let file = std::fs::File::create(&target).expect("target");
  file
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1))
    .expect("old target time");
  std::os::unix::fs::symlink(&target, &path).expect("link");
  let manager = Arc::new(WatcherManager::new(FsWatcherIgnored::default(), true));
  manager
    .update(Default::default(), (vec![root], vec![]), Default::default())
    .await
    .expect("register context");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  manager.reset();
  manager
    .update((vec![path], vec![]), Default::default(), Default::default())
    .await
    .expect("attach link");
  let start = SystemTime::now() - Duration::from_secs(1);
  assert!(
    manager.scan_registered(start).await.is_empty(),
    "link mtime must not stand in for target mtime"
  );
  std::fs::remove_file(&target).expect("remove test target");
  assert!(
    manager.scan_registered(start).await.is_empty(),
    "a dangling registered link still exists"
  );
}
