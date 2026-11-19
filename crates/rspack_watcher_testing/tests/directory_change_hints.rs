use std::{
  fs,
  iter::{empty, once},
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher,
  testing::{disk_event, manual_watcher},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, PartialEq, Eq)]
enum Notice {
  Change(String),
  Remove(String),
  Batch(FxHashSet<String>, FxHashSet<String>),
}

struct Handler(UnboundedSender<Notice>);

impl EventHandler for Handler {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    self.0.send(Notice::Change(path)).expect("collect change");
    Ok(())
  }

  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    self.0.send(Notice::Remove(path)).expect("collect removal");
    Ok(())
  }
}

impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, changed: FxHashSet<String>, removed: FxHashSet<String>) {
    self
      .0
      .send(Notice::Batch(changed, removed))
      .expect("collect batch");
  }
}

async fn expect_batch(
  notices: &mut UnboundedReceiver<Notice>,
  changed: &[&InternedPath],
  removed: &[&InternedPath],
) -> Vec<Notice> {
  let names = |paths: &[&InternedPath]| {
    paths
      .iter()
      .map(|p| p.to_string_lossy().into_owned())
      .collect()
  };
  tokio::time::timeout(Duration::from_secs(5), async {
    let mut raw = Vec::new();
    loop {
      match notices.recv().await.expect("watcher is open") {
        Notice::Batch(changes, removals) => {
          assert_eq!((changes, removals), (names(changed), names(removed)));
          return raw;
        }
        notice => raw.push(notice),
      }
    }
  })
  .await
  .expect("receive aggregate for a single disk event")
}

async fn watch(
  root: &std::path::Path,
  files: &[&InternedPath],
  contexts: &[&InternedPath],
  missing: &[&InternedPath],
) -> (FsWatcher, UnboundedReceiver<Notice>) {
  let watcher = manual_watcher();
  let (tx, mut notices) = unbounded_channel();
  let start = SystemTime::now() + Duration::from_secs(60);
  // The scanner reports this absent file even with a future start time.
  let ready = InternedPath::from(root.join("absent-on-startup.js"));
  watcher
    .watch(
      (
        files.iter().copied().cloned().chain(once(ready.clone())),
        empty(),
      ),
      (contexts.iter().copied().cloned(), empty()),
      (missing.iter().copied().cloned(), empty()),
      start,
      Box::new(Handler(tx.clone())),
      Box::new(Handler(tx)),
    )
    .await;
  expect_batch(&mut notices, &[], &[&ready]).await;
  (watcher, notices)
}

fn write_at(path: &InternedPath, time: SystemTime) {
  fs::File::create(path)
    .expect("create fixture file")
    .set_modified(time)
    .expect("set fixture mtime");
}

#[tokio::test]
async fn parent_change_reports_a_deleted_file_once() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let src = InternedPath::from(root.join("src"));
  let file = InternedPath::from(src.join("a.js"));
  let missing = InternedPath::from(src.join("never-existed.js"));
  fs::create_dir(&src).expect("create src");
  write_at(&file, SystemTime::now());
  let (watcher, mut notices) = watch(&root, &[&file], &[&src], &[&missing]).await;

  fs::remove_file(&file).expect("remove a.js");
  disk_event(&watcher, &src, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&src], &[&file]).await;
  let removal = Notice::Remove(file.to_string_lossy().into_owned());
  assert_eq!(raw.iter().filter(|n| **n == removal).count(), 1);
  assert!(raw.contains(&Notice::Change(file.to_string_lossy().into_owned())));

  disk_event(&watcher, &file, FsEventKind::Remove);
  disk_event(&watcher, &src, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&src], &[]).await;
  assert!(!raw.iter().any(|n| matches!(n, Notice::Remove(_))));
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn ancestor_change_reports_files_after_their_directory_disappears() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let src = InternedPath::from(root.join("src"));
  let sub = InternedPath::from(src.join("sub"));
  fs::create_dir_all(&sub).expect("create directory");
  let a = InternedPath::from(sub.join("a.js"));
  let b = InternedPath::from(sub.join("b.js"));
  write_at(&a, SystemTime::now());
  write_at(&b, SystemTime::now());
  let (watcher, mut notices) = watch(&root, &[&a, &b], &[], &[]).await;

  fs::remove_dir_all(&sub).expect("remove directory with both files");
  disk_event(&watcher, &src, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[], &[&a, &b]).await;
  assert_eq!(raw.len(), 2);
  for path in [a, b] {
    assert!(raw.contains(&Notice::Remove(path.to_string_lossy().into_owned())));
  }
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn moved_directory_reports_a_missing_file_with_an_old_mtime() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let src = InternedPath::from(root.join("src"));
  let pkg = InternedPath::from(src.join("pkg"));
  let file = InternedPath::from(pkg.join("a.js"));
  let staging = root.join("staging");
  fs::create_dir(&src).expect("create src");
  fs::create_dir(&staging).expect("create staging directory");
  write_at(
    &InternedPath::from(staging.join("a.js")),
    SystemTime::now() - Duration::from_secs(3600),
  );
  let (watcher, mut notices) = watch(&root, &[], &[&src], &[&file]).await;

  fs::rename(staging, &pkg).expect("move pkg into src");
  disk_event(&watcher, &pkg, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&src, &file], &[]).await;
  assert!(raw.contains(&Notice::Change(file.to_string_lossy().into_owned())));

  disk_event(&watcher, &pkg, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&src], &[]).await;
  assert!(!raw.contains(&Notice::Change(file.to_string_lossy().into_owned())));
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn an_existing_missing_dependency_waits_for_its_own_event() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let src = InternedPath::from(root.join("src"));
  let file = InternedPath::from(src.join("existing.js"));
  fs::create_dir(&src).expect("create src");
  write_at(&file, SystemTime::now() - Duration::from_secs(3600));
  let (watcher, mut notices) = watch(&root, &[], &[&src], &[&file]).await;

  disk_event(&watcher, &src, FsEventKind::Change);
  expect_batch(&mut notices, &[&src], &[]).await;
  disk_event(&watcher, &file, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&src, &file], &[]).await;
  assert_eq!(raw, [Notice::Change(file.to_string_lossy().into_owned())]);
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn nested_contexts_emit_one_immediate_change_and_aggregate_both_directories() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let src = InternedPath::from(root.join("src"));
  let sub = InternedPath::from(src.join("sub"));
  let file = InternedPath::from(sub.join("a.js"));
  fs::create_dir_all(&sub).expect("create nested contexts");
  let old = SystemTime::now() - Duration::from_secs(3600);
  write_at(&file, old);
  let (watcher, mut notices) = watch(&root, &[], &[&src, &sub], &[]).await;

  write_at(&file, old + Duration::from_secs(10));
  disk_event(&watcher, &file, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&src, &sub], &[]).await;
  assert_eq!(raw, [Notice::Change(file.to_string_lossy().into_owned())]);
  watcher.close().await.expect("close watcher");
}

async fn check_directory_replaced_by_file(disk_hint: bool) {
  use rspack_watcher::{TimeInfoEntry, testing::check_subscriptions};

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let ctx = InternedPath::from(root.join("ctx"));
  let old_child = InternedPath::from(ctx.join("old.js"));
  let marker = InternedPath::from(root.join("marker"));
  fs::create_dir(&ctx).expect("create directory");
  fs::create_dir(&marker).expect("create marker directory");
  write_at(&old_child, SystemTime::now());
  let (watcher, mut notices) = watch(&root, &[], &[&ctx, &marker], &[]).await;

  fs::remove_dir_all(&ctx).expect("remove original directory");
  write_at(&ctx, SystemTime::now());
  if disk_hint {
    disk_event(&watcher, &ctx, FsEventKind::Change);
  } else {
    check_subscriptions(&watcher).await;
  }
  let raw = expect_batch(&mut notices, &[], &[&ctx]).await;
  assert_eq!(raw, [Notice::Remove(ctx.to_string_lossy().into_owned())]);
  let (files, dirs) = watcher.collect_time_info_entries();
  for entries in [&files, &dirs] {
    assert!(entries.contains(&(ctx.to_string_lossy().into_owned(), TimeInfoEntry::Null)));
    assert!(
      !entries
        .iter()
        .any(|(name, _)| name == &old_child.to_string_lossy())
    );
  }

  check_subscriptions(&watcher).await;
  disk_event(&watcher, &marker, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&marker], &[]).await;
  assert_eq!(raw, [Notice::Change(marker.to_string_lossy().into_owned())]);

  fs::remove_file(&ctx).expect("remove replacement file");
  fs::create_dir(&ctx).expect("restore directory");
  let new_child = InternedPath::from(ctx.join("new.js"));
  write_at(&new_child, SystemTime::now());
  check_subscriptions(&watcher).await;
  let raw = expect_batch(&mut notices, &[&ctx], &[]).await;
  assert_eq!(raw, [Notice::Change(ctx.to_string_lossy().into_owned())]);
  assert!(
    watcher
      .collect_time_info_entries()
      .0
      .iter()
      .any(|(name, entry)| name == &new_child.to_string_lossy()
        && matches!(entry, TimeInfoEntry::Entry { .. }))
  );
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn active_check_detects_when_a_watched_directory_becomes_a_file() {
  check_directory_replaced_by_file(false).await;
}

#[tokio::test]
async fn named_event_detects_when_a_watched_directory_becomes_a_file() {
  check_directory_replaced_by_file(true).await;
}

#[tokio::test]
async fn replacing_a_directory_keeps_the_file_subscription_on_the_same_path() {
  use rspack_watcher::{
    TimeInfoEntry,
    testing::{check_subscriptions, watch_directory_subscriptions},
  };

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let path = InternedPath::from(root.join("entry"));
  let directory_request = InternedPath::from(root.join("directory-request"));
  fs::create_dir(&path).expect("create directory");
  let (watcher, _initial) = watch(&root, &[&path], &[], &[]).await;
  let (tx, mut notices) = unbounded_channel();
  watch_directory_subscriptions(
    &watcher,
    &[(path.clone(), directory_request.clone())],
    &[],
    SystemTime::now() + Duration::from_secs(60),
    Handler(tx.clone()),
    Handler(tx),
  )
  .await;

  fs::remove_dir(&path).expect("remove original directory");
  write_at(&path, SystemTime::now() - Duration::from_secs(3600));
  check_subscriptions(&watcher).await;
  let raw = expect_batch(&mut notices, &[&path], &[&directory_request]).await;
  assert_eq!(raw.len(), 2);
  assert!(raw.contains(&Notice::Remove(
    directory_request.to_string_lossy().into_owned()
  )));
  assert!(raw.contains(&Notice::Change(path.to_string_lossy().into_owned())));
  let (files, dirs) = watcher.collect_time_info_entries();
  assert!(
    files
      .iter()
      .any(|(name, entry)| name == &path.to_string_lossy()
        && matches!(entry, TimeInfoEntry::Entry { .. }))
  );
  assert!(dirs.contains(&(path.to_string_lossy().into_owned(), TimeInfoEntry::Null)));
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn a_single_hint_for_the_removed_directory_reports_its_registered_file() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let dir = InternedPath::from(root.join("src"));
  let file = InternedPath::from(dir.join("a.js"));
  fs::create_dir(&dir).expect("create directory");
  write_at(&file, SystemTime::now());
  let (watcher, mut notices) = watch(&root, &[&file], &[], &[]).await;
  fs::remove_dir_all(&dir).expect("remove directory with the watched file");
  disk_event(&watcher, &dir, FsEventKind::Remove);
  let raw = expect_batch(&mut notices, &[], &[&file]).await;
  assert_eq!(raw, [Notice::Remove(file.to_string_lossy().into_owned())]);
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn overlapping_contexts_preserve_each_distinct_directory_alias() {
  use rspack_watcher::testing::watch_directory_subscriptions;

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let src = InternedPath::from(root.join("src"));
  let sub = InternedPath::from(src.join("sub"));
  let a = InternedPath::from(root.join("alias-a"));
  let b = InternedPath::from(root.join("alias-b"));
  let file = InternedPath::from(sub.join("file.js"));
  fs::create_dir_all(&sub).expect("create directories");
  let old = SystemTime::now() - Duration::from_secs(3600);
  write_at(&file, old);
  let (watcher, _initial) = watch(&root, &[&file], &[&src, &sub], &[]).await;
  let (tx, mut notices) = unbounded_channel();
  watch_directory_subscriptions(
    &watcher,
    &[
      (src.clone(), a.clone()),
      (sub.clone(), a.clone()),
      (sub.clone(), b.clone()),
    ],
    &[],
    SystemTime::now() + Duration::from_secs(60),
    Handler(tx.clone()),
    Handler(tx),
  )
  .await;

  write_at(&file, old + Duration::from_secs(10));
  disk_event(&watcher, &file, FsEventKind::Change);
  let raw = expect_batch(&mut notices, &[&file, &src, &sub, &a, &b], &[]).await;
  let name = file.to_string_lossy();
  let changes: [Notice; 3] = std::array::from_fn(|_| Notice::Change(name.to_string()));
  assert_eq!(raw, changes);

  fs::remove_file(&file).expect("remove file");
  disk_event(&watcher, &file, FsEventKind::Remove);
  let raw = expect_batch(&mut notices, &[&src, &sub, &a, &b], &[&file]).await;
  assert_eq!(raw.first(), Some(&Notice::Remove(name.to_string())));
  assert_eq!(raw[1..], changes);
  watcher.close().await.expect("close watcher");
}
