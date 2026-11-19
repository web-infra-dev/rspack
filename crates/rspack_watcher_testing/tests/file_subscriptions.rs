use std::{
  fs,
  io::Write,
  iter::{empty, once},
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, TimeInfoEntry,
  testing::{check_subscriptions, disk_event, manual_watcher, watch_file_subscriptions},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

struct Changes(UnboundedSender<String>);

impl EventHandler for Changes {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    self.0.send(path).expect("collect change");
    Ok(())
  }

  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    self
      .0
      .send(format!("removed {path}"))
      .expect("collect removal");
    Ok(())
  }
}

type Batch = (FxHashSet<String>, FxHashSet<String>);

struct Batches(UnboundedSender<Batch>);

impl EventAggregateHandler for Batches {
  fn on_event_handle(&self, changed: FxHashSet<String>, removed: FxHashSet<String>) {
    self.0.send((changed, removed)).expect("collect batch");
  }
}

async fn assert_batch(
  batches: &mut UnboundedReceiver<Batch>,
  changed: &[&InternedPath],
  removed: &[&InternedPath],
) {
  let batch = tokio::time::timeout(Duration::from_secs(5), batches.recv())
    .await
    .expect("receive batch")
    .expect("batch handler still active");
  let names = |paths: &[&InternedPath]| {
    paths
      .iter()
      .map(|path| path.to_string_lossy().into_owned())
      .collect()
  };
  assert_eq!(batch, (names(changed), names(removed)));
}

fn assert_immediate_changes(raw: &mut UnboundedReceiver<String>, expected: &[&InternedPath]) {
  let mut actual: Vec<_> = std::iter::from_fn(|| raw.try_recv().ok()).collect();
  let mut expected: Vec<_> = expected.iter().map(|path| path.to_string_lossy()).collect();
  actual.sort_unstable();
  expected.sort_unstable();
  assert_eq!(actual, expected);
}

fn write_version(path: &InternedPath, version: u64) {
  let mut file = fs::File::create(path).expect("open observed file");
  writeln!(file, "{version}").expect("write observed file");
  file
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000 + version))
    .expect("set distinct, aged mtime");
}

#[tokio::test]
async fn shared_file_notifies_each_subscriber_on_disk_changes_and_rescans() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let source = InternedPath::from(root.join("shared.js"));
  let a = InternedPath::from(root.join("a.js"));
  let b = InternedPath::from(root.join("b.js"));
  let bindings = [(source.clone(), a.clone()), (source.clone(), b.clone())];
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  let watch = |added, removed| {
    watch_file_subscriptions(
      &watcher,
      added,
      removed,
      Batches(batch_tx.clone()),
      Changes(raw_tx.clone()),
    )
  };
  write_version(&source, 0);
  watch(&bindings, &[]).await;

  write_version(&source, 1);
  disk_event(&watcher, &source, FsEventKind::Change);
  assert_batch(&mut batches, &[&a, &b], &[]).await;
  assert_immediate_changes(&mut raw, &[&a, &b]);

  write_version(&source, 2);
  check_subscriptions(&watcher).await;
  assert_batch(&mut batches, &[&a, &b], &[]).await;
  assert_immediate_changes(&mut raw, &[&a, &b]);
  let (times, _) = watcher.collect_time_info_entries();
  assert!(
    matches!(
      times.as_slice(),
      [(path, TimeInfoEntry::Entry { timestamp: 1_600_000_002_000, .. })]
        if path == &source.to_string_lossy()
    ),
    "expected the observed file's mtime, got {times:?}"
  );

  watch(&[], &bindings[..1]).await;
  assert_eq!(watcher.collect_time_info_entries().0, times);
  write_version(&source, 3);
  disk_event(&watcher, &source, FsEventKind::Change);
  assert_batch(&mut batches, &[&b], &[]).await;
  assert_immediate_changes(&mut raw, &[&b]);

  watch(&[], &bindings[1..]).await;
  assert!(watcher.collect_time_info_entries().0.is_empty());
  write_version(&source, 4);
  disk_event(&watcher, &source, FsEventKind::Change);
  assert!(
    tokio::time::timeout(Duration::from_millis(200), batches.recv())
      .await
      .is_err()
  );
  assert_immediate_changes(&mut raw, &[]);
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn one_request_keeps_the_mtime_of_each_observed_file() {
  use rspack_watcher::testing::watch_missing_subscriptions;

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let first = InternedPath::from(root.join("first.js"));
  let second = InternedPath::from(root.join("second.js"));
  let requested = InternedPath::from(root.join("requested.js"));
  write_version(&first, 1);
  write_version(&second, 2);
  let watcher = manual_watcher();
  let (raw, _raw_rx) = unbounded_channel();
  let (batches, _batch_rx) = unbounded_channel();
  watch_file_subscriptions(
    &watcher,
    &[
      (first.clone(), requested.clone()),
      (second.clone(), requested),
    ],
    &[],
    Batches(batches.clone()),
    Changes(raw.clone()),
  )
  .await;
  watch_missing_subscriptions(
    &watcher,
    &[(first.clone(), InternedPath::from(root.join("missing.js")))],
    &[],
    Batches(batches),
    Changes(raw),
  )
  .await;

  let (mut times, _) = watcher.collect_time_info_entries();
  times.sort_by(|a, b| a.0.cmp(&b.0));
  assert_eq!(times.len(), 2);
  for ((path, entry), (observed, mtime)) in times
    .iter()
    .zip([(first, 1_600_000_001_000), (second, 1_600_000_002_000)])
  {
    assert_eq!(path.as_str(), observed.to_string_lossy());
    assert!(matches!(entry, TimeInfoEntry::Entry { timestamp, .. } if *timestamp == mtime));
  }
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn creating_missing_parent_directories_waits_for_the_registered_file() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let parent = InternedPath::from(root.join("missing"));
  let pkg = InternedPath::from(parent.join("pkg"));
  let file = InternedPath::from(pkg.join("index.js"));
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  watcher
    .watch(
      (empty(), empty()),
      (empty(), empty()),
      (once(file.clone()), empty()),
      SystemTime::now(),
      Box::new(Batches(batch_tx)),
      Box::new(Changes(raw_tx)),
    )
    .await;

  for dir in [&parent, &pkg] {
    fs::create_dir(dir).expect("create missing parent");
    disk_event(&watcher, dir, FsEventKind::Create);
    check_subscriptions(&watcher).await;
  }
  assert!(
    tokio::time::timeout(Duration::from_millis(150), batches.recv())
      .await
      .is_err()
  );
  assert_immediate_changes(&mut raw, &[]);

  write_version(&file, 1);
  disk_event(&watcher, &file, FsEventKind::Create);
  assert_batch(&mut batches, &[&file], &[]).await;
  assert_immediate_changes(&mut raw, &[&file]);
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn removing_the_file_subscription_leaves_its_context_active() {
  let temp = tempfile::tempdir().expect("create fixture");
  let dir = InternedPath::from(temp.path().canonicalize().expect("resolve fixture"));
  let file = InternedPath::from(dir.join("child.js"));
  write_version(&file, 0);
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  let start = SystemTime::now() + Duration::from_secs(30);
  watcher
    .watch(
      (once(file.clone()), empty()),
      (once(dir.clone()), empty()),
      (empty(), empty()),
      start,
      Box::new(Batches(batch_tx.clone())),
      Box::new(Changes(raw_tx.clone())),
    )
    .await;
  watcher
    .watch(
      (empty(), once(file.clone())),
      (empty(), empty()),
      (empty(), empty()),
      start,
      Box::new(Batches(batch_tx)),
      Box::new(Changes(raw_tx)),
    )
    .await;

  write_version(&file, 1);
  check_subscriptions(&watcher).await;
  assert_batch(&mut batches, &[&dir], &[]).await;
  assert_immediate_changes(&mut raw, &[&file]);
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn missing_subscribers_share_a_watched_path() {
  use rspack_watcher::testing::watch_missing_subscriptions;

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let parent = InternedPath::from(root.join("not_exist"));
  let watched = InternedPath::from(parent.join("index.js"));
  let a = InternedPath::from(root.join("link-a/not_exist/index.js"));
  let b = InternedPath::from(root.join("link-b/not_exist/index.js"));
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  let subscribe = |added, removed| {
    watch_missing_subscriptions(
      &watcher,
      added,
      removed,
      Batches(batch_tx.clone()),
      Changes(raw_tx.clone()),
    )
  };
  let bindings = [(watched.clone(), a.clone()), (watched.clone(), b.clone())];
  subscribe(&bindings, &[]).await;
  let observed = watched.to_string_lossy().into_owned();
  let (times, _) = watcher.collect_time_info_entries();
  assert_eq!(times, [(observed.clone(), TimeInfoEntry::Null)]);

  fs::create_dir(&parent).expect("create missing parent");
  write_version(&watched, 1);
  disk_event(&watcher, &parent, FsEventKind::Create);
  assert_batch(&mut batches, &[&a, &b], &[]).await;
  assert_immediate_changes(&mut raw, &[&a, &b]);

  subscribe(&[], &bindings[..1]).await;
  let (times, _) = watcher.collect_time_info_entries();
  assert!(matches!(times.as_slice(), [(path, TimeInfoEntry::Entry { .. })] if path == &observed));
  fs::remove_file(&watched).expect("remove watched file");
  check_subscriptions(&watcher).await;
  assert_batch(&mut batches, &[], &[&b]).await;
  subscribe(&[], &bindings[1..]).await;
  assert!(watcher.collect_time_info_entries().0.is_empty());
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn child_event_notifies_the_requested_missing_directory() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let watched = InternedPath::from(root.join("disk"));
  let requested = InternedPath::from(root.join("link"));
  let watcher = manual_watcher();
  let (batch_tx, mut batches) = unbounded_channel();
  let (raw_tx, _raw) = unbounded_channel();
  rspack_watcher::testing::watch_missing_subscriptions(
    &watcher,
    &[(watched.clone(), requested.clone())],
    &[],
    Batches(batch_tx),
    Changes(raw_tx),
  )
  .await;

  fs::create_dir(&watched).expect("create watched directory");
  let child = InternedPath::from(watched.join("index.js"));
  write_version(&child, 1);
  disk_event(&watcher, &child, FsEventKind::Create);
  assert_batch(&mut batches, &[&requested], &[]).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn directory_subscribers_share_a_watched_directory() {
  use rspack_watcher::testing::watch_directory_subscriptions;

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let disk = InternedPath::from(root.join("disk"));
  let a = InternedPath::from(root.join("a"));
  let b = InternedPath::from(root.join("b"));
  let child = InternedPath::from(disk.join("child.js"));
  let bindings = [(disk.clone(), a.clone()), (disk.clone(), b.clone())];
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  let watch = |added, removed| {
    watch_directory_subscriptions(
      &watcher,
      added,
      removed,
      SystemTime::now() + Duration::from_secs(30),
      Batches(batch_tx.clone()),
      Changes(raw_tx.clone()),
    )
  };
  fs::create_dir(&disk).expect("create watched directory");
  write_version(&child, 0);
  watch(&bindings, &[]).await;

  disk_event(&watcher, &disk, FsEventKind::Change);
  assert_batch(&mut batches, &[&a, &b], &[]).await;
  assert_immediate_changes(&mut raw, &[&a, &b]);

  write_version(&child, 1);
  disk_event(&watcher, &child, FsEventKind::Change);
  assert_batch(&mut batches, &[&a, &b], &[]).await;
  assert_immediate_changes(&mut raw, &[&child, &child]);

  let (files, dirs) = watcher.collect_time_info_entries();
  let [(path, TimeInfoEntry::OnlySafeTimeEntry { safe_time })] = dirs.as_slice() else {
    panic!("expected one observed directory timestamp, got {dirs:?}");
  };
  assert_eq!(path.as_str(), disk.to_string_lossy());
  assert!(*safe_time > 0);
  assert_eq!(files.len(), 2);
  assert!(files.contains(&(path.clone(), TimeInfoEntry::ExistenceOnlyTimeEntry)));
  assert!(files.iter().any(
    |(p, entry)| p == &child.to_string_lossy() && matches!(entry, TimeInfoEntry::Entry { .. })
  ));

  watch(&[], &bindings[..1]).await;
  fs::remove_dir_all(&disk).expect("remove watched directory");
  disk_event(&watcher, &disk, FsEventKind::Remove);
  assert_batch(&mut batches, &[], &[&b]).await;
  assert_eq!(raw.try_recv().unwrap(), format!("removed {}", b.display()));
  watch(&[], &bindings[1..]).await;
  assert!(watcher.collect_time_info_entries().1.is_empty());
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn scanned_directories_route_descendants_and_release_their_state() {
  use rspack_watcher::testing::watch_directory_subscriptions;

  let temp = tempfile::tempdir().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let outer = InternedPath::from(base.join("b"));
  let inner = InternedPath::from(outer.join("c"));
  let alias = InternedPath::from(base.join("src").join("x"));
  let suffix = std::path::Path::new("d").join("e").join("f.js");
  let file = InternedPath::from(inner.join(&suffix));
  let outer_binding = [(outer.clone(), outer.clone())];
  let inner_binding = [(inner.clone(), alias.clone())];
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  let watch = |added, removed| {
    watch_directory_subscriptions(
      &watcher,
      added,
      removed,
      SystemTime::now() + Duration::from_secs(60),
      Batches(batch_tx.clone()),
      Changes(raw_tx.clone()),
    )
  };

  fs::create_dir(&outer).expect("create outer context");
  watch(&outer_binding, &[]).await;
  let outer_count = watcher.index_sizes().1;
  fs::create_dir_all(file.parent().expect("parent")).expect("create nested directories");
  write_version(&file, 0);
  watch(&inner_binding, &[]).await;
  assert_eq!(
    watcher.index_sizes().1,
    outer_count + 3,
    "scan creates c, d and e"
  );

  write_version(&file, 1);
  disk_event(&watcher, &file, FsEventKind::Change);
  assert_batch(&mut batches, &[&outer, &alias], &[]).await;
  assert_immediate_changes(&mut raw, &[&file, &file]);

  watch(&[], &outer_binding).await;
  fs::remove_file(&file).expect("remove file");
  disk_event(&watcher, &file, FsEventKind::Remove);
  assert_batch(&mut batches, &[&alias], &[]).await;
  assert_immediate_changes(&mut raw, &[&file]);

  fs::remove_dir_all(inner.join("d")).expect("remove scanned subtree");
  disk_event(
    &watcher,
    &InternedPath::from(inner.join("d")),
    FsEventKind::Remove,
  );
  assert_batch(&mut batches, &[&alias], &[]).await;
  assert_eq!(
    watcher.index_sizes().1,
    outer_count + 1,
    "d and e were released"
  );
  watch(&[], &inner_binding).await;
  assert_eq!(watcher.index_sizes().1, 0);
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn missing_subscription_keeps_child_events_after_directory_times_are_released() {
  let temp = tempfile::tempdir().expect("create fixture");
  let dir = InternedPath::from(temp.path().canonicalize().expect("resolve fixture"));
  let child = InternedPath::from(dir.join("child.js"));
  write_version(&child, 0);
  let watcher = manual_watcher();
  let (batch_tx, mut batches) = unbounded_channel();
  let (raw_tx, _raw) = unbounded_channel();
  let start = SystemTime::now() + Duration::from_secs(60);
  let has_child_time = || {
    watcher
      .collect_time_info_entries()
      .0
      .iter()
      .any(|(path, _)| path == &child.to_string_lossy())
  };
  watcher
    .watch(
      (empty(), empty()),
      (once(dir.clone()), empty()),
      (once(dir.clone()), empty()),
      start,
      Box::new(Batches(batch_tx.clone())),
      Box::new(Changes(raw_tx.clone())),
    )
    .await;
  assert!(has_child_time());

  watcher
    .watch(
      (empty(), empty()),
      (empty(), once(dir.clone())),
      (empty(), empty()),
      start,
      Box::new(Batches(batch_tx)),
      Box::new(Changes(raw_tx)),
    )
    .await;
  assert!(!has_child_time());
  write_version(&child, 1);
  disk_event(&watcher, &child, FsEventKind::Change);
  assert_batch(&mut batches, &[&dir], &[]).await;
  assert!(!has_child_time());
  watcher.close().await.expect("close watcher");
}
