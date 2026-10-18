use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
  },
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use tokio::{sync::Notify, time::timeout};

use crate::{FsWatcherIgnored, dirwatch::WatcherManager};

const DEADLINE: Duration = Duration::from_secs(5);

async fn register(manager: &WatcherManager, context: &InternedPath, added: bool) {
  manager.reset();
  let directories = if added {
    (vec![context.clone()], Vec::new())
  } else {
    (Vec::new(), vec![context.clone()])
  };
  manager
    .update(Default::default(), directories, Default::default())
    .await
    .expect("update context registration");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scans_of_one_directory_do_not_overlap() {
  let temp = tempfile::tempdir().expect("create directory");
  let context = InternedPath::from(temp.path());
  let file = context.join("entry.js");
  std::fs::write(&file, "entry").expect("write entry");
  let entered = Arc::new(Notify::new());
  let release = Arc::new(Notify::new());
  let overlap = Arc::new(Notify::new());
  let active = Arc::new(AtomicUsize::new(0));
  let first = Arc::new(AtomicBool::new(true));
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let (entered, release, overlap, active) = (
      Arc::clone(&entered),
      Arc::clone(&release),
      Arc::clone(&overlap),
      Arc::clone(&active),
    );
    move |path| {
      let is_file = std::path::Path::new(&path) == file.as_ref() as &std::path::Path;
      let first = is_file && first.swap(false, Ordering::SeqCst);
      let (entered, release, overlap, active) = (
        Arc::clone(&entered),
        Arc::clone(&release),
        Arc::clone(&overlap),
        Arc::clone(&active),
      );
      Box::pin(async move {
        if is_file {
          if active.fetch_add(1, Ordering::SeqCst) != 0 {
            overlap.notify_one();
          }
          if first {
            entered.notify_one();
            release.notified().await;
          }
          active.fetch_sub(1, Ordering::SeqCst);
        }
        false
      })
    }
  }));
  let manager = Arc::new(WatcherManager::new(ignored, false));
  register(&manager, &context, true).await;
  let scan = |manager: Arc<WatcherManager>| {
    tokio::spawn(async move { manager.scan_contexts(SystemTime::now()).await })
  };
  let first_scan = scan(Arc::clone(&manager));
  timeout(DEADLINE, entered.notified())
    .await
    .expect("first scan reached predicate");
  let second_scan = scan(Arc::clone(&manager));
  let overlapped = timeout(Duration::from_millis(100), overlap.notified())
    .await
    .is_ok();
  release.notify_one();
  timeout(DEADLINE, async {
    first_scan.await.expect("first scan completed");
    second_scan.await.expect("second scan completed");
  })
  .await
  .expect("both scans completed");
  assert!(
    !overlapped,
    "two scans entered the same directory concurrently"
  );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_old_scan_cannot_write_into_a_recreated_watcher() {
  let temp = tempfile::tempdir().expect("create directory");
  let context = InternedPath::from(temp.path());
  let file = InternedPath::from(context.join("entry.js"));
  std::fs::write(&file, "entry").expect("write entry");
  let entered = Arc::new(Notify::new());
  let release = Arc::new(Notify::new());
  let first = Arc::new(AtomicBool::new(true));
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let (file, entered, release) = (file.clone(), Arc::clone(&entered), Arc::clone(&release));
    move |path| {
      let is_file = std::path::Path::new(&path) == file.as_ref() as &std::path::Path;
      let first = is_file && first.swap(false, Ordering::SeqCst);
      let (entered, release) = (Arc::clone(&entered), Arc::clone(&release));
      Box::pin(async move {
        if first {
          entered.notify_one();
          release.notified().await;
        }
        is_file && !first
      })
    }
  }));
  let manager = Arc::new(WatcherManager::new(ignored, false));
  register(&manager, &context, true).await;
  let old_scan = tokio::spawn({
    let manager = Arc::clone(&manager);
    async move { manager.scan_contexts(SystemTime::now()).await }
  });
  timeout(DEADLINE, entered.notified())
    .await
    .expect("old scan reached predicate");
  register(&manager, &context, false).await;
  register(&manager, &context, true).await;
  timeout(DEADLINE, manager.scan_contexts(SystemTime::now()))
    .await
    .expect("new watcher scans independently");
  assert!(manager.entry(&file).is_none(), "new watcher ignores entry");
  release.notify_one();
  timeout(DEADLINE, old_scan)
    .await
    .expect("old scan returned")
    .expect("old scan completed");
  assert!(
    manager.entry(&file).is_none(),
    "old scan populated the replacement watcher"
  );
}

#[cfg(unix)]
#[tokio::test]
async fn switching_symlink_target_during_scan_retries_the_directory() {
  let temp = tempfile::tempdir().expect("directory");
  let old = temp.path().join("old");
  let new = temp.path().join("new");
  std::fs::create_dir(&old).expect("old target");
  std::fs::create_dir(&new).expect("new target");
  std::fs::write(old.join("old.js"), "old").expect("old entry");
  std::fs::write(new.join("new.js"), "new").expect("new entry");
  let context = InternedPath::from(temp.path().join("link"));
  std::os::unix::fs::symlink(&old, &context).expect("link");
  let release = Arc::new(Notify::new());
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let release = Arc::clone(&release);
    move |path| {
      let release = Arc::clone(&release);
      Box::pin(async move {
        if path.ends_with("old.js") {
          release.notified().await;
        }
        false
      })
    }
  }));
  let manager = WatcherManager::new(ignored, true);
  register(&manager, &context, true).await;
  let scan = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(scan);
  assert!(futures::poll!(&mut scan).is_pending());
  std::fs::remove_file(&context).expect("unlink");
  std::os::unix::fs::symlink(&new, &context).expect("replace target");
  release.notify_one();
  let changes = timeout(DEADLINE, scan).await.expect("scan completes");
  let new_entry = InternedPath::from(context.join("new.js"));
  assert!(
    changes.contains(&new_entry),
    "completed scan must include the current target: {changes:?}"
  );
  assert!(manager.entry(&new_entry).expect("new entry").time.is_some());
  assert!(
    manager
      .entry(&InternedPath::from(context.join("old.js")))
      .is_none()
  );
}

#[cfg(unix)]
async fn failed_scan_is_not_reusable(follow_symlinks: bool) {
  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path().join("context"));
  let saved = temp.path().join("saved");
  std::fs::create_dir(&context).expect("context");
  let manager = WatcherManager::new(FsWatcherIgnored::default(), follow_symlinks);
  register(&manager, &context, true).await;
  std::fs::rename(&context, &saved).expect("save directory");
  std::os::unix::fs::symlink(&context, &context).expect("loop");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  let (finished, invalidated) = {
    let watcher = manager.watchers.get(&context).expect("watcher");
    (watcher.initial_scan_finished, watcher.scan_invalidated)
  };
  std::fs::remove_file(&context).expect("unlink loop");
  std::fs::rename(&saved, &context).expect("restore directory");
  assert!(
    invalidated,
    "a failed scan is not a reusable directory snapshot"
  );
  assert!(
    finished.is_some(),
    "failed attempt still finishes the initial scan"
  );
  let file = InternedPath::from(context.join("entry.js"));
  std::fs::write(&file, "entry").expect("entry");
  assert!(
    manager
      .scan_contexts(SystemTime::UNIX_EPOCH)
      .await
      .contains(&file)
  );
  assert!(
    manager
      .watchers
      .get(&context)
      .expect("watcher")
      .initial_scan_finished
      .is_some()
  );
}

#[cfg(unix)]
#[tokio::test]
async fn readdir_failure_invalidates_scan_cache() {
  failed_scan_is_not_reusable(false).await;
}

#[cfg(unix)]
#[tokio::test]
async fn canonicalization_failure_invalidates_scan_cache() {
  failed_scan_is_not_reusable(true).await;
}
