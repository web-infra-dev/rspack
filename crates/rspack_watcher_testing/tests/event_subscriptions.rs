use std::{
  fs,
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind,
  testing::{DirectoryAnalyzer, disk_event, manual_watcher, watch_subscriptions},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Immediate {
  Change(String),
  Remove(String),
}

enum Notice {
  Immediate(Immediate),
  Batch(FxHashSet<String>, FxHashSet<String>),
}

struct Handler(UnboundedSender<Notice>);

impl EventHandler for Handler {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    self
      .0
      .send(Notice::Immediate(Immediate::Change(path)))
      .expect("collect change");
    Ok(())
  }

  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    self
      .0
      .send(Notice::Immediate(Immediate::Remove(path)))
      .expect("collect removal");
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

fn write(path: &InternedPath, version: u64) {
  fs::File::create(path)
    .expect("create fixture file")
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000 + version))
    .expect("set distinct file mtime");
}

fn name(path: &InternedPath) -> String {
  path.to_string_lossy().into_owned()
}

async fn expect_batch(
  notices: &mut UnboundedReceiver<Notice>,
  changes: &[&InternedPath],
  removals: &[&InternedPath],
  raw_changes: &[&InternedPath],
  raw_removals: &[&InternedPath],
) {
  let mut expected: Vec<_> = raw_changes
    .iter()
    .map(|p| Immediate::Change(name(p)))
    .chain(raw_removals.iter().map(|p| Immediate::Remove(name(p))))
    .collect();
  expected.sort();
  tokio::time::timeout(Duration::from_secs(5), async {
    let mut raw = Vec::new();
    loop {
      match notices.recv().await.expect("watcher is open") {
        Notice::Immediate(event) => raw.push(event),
        Notice::Batch(changed, removed) => {
          raw.sort();
          assert_eq!(raw, expected);
          assert_eq!(
            changed,
            changes.iter().map(|p| name(p)).collect::<FxHashSet<_>>()
          );
          assert_eq!(
            removed,
            removals.iter().map(|p| name(p)).collect::<FxHashSet<_>>()
          );
          return;
        }
      }
    }
  })
  .await
  .expect("receive event batch");
}

#[tokio::test]
async fn one_disk_change_notifies_each_remaining_file_subscription() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let source = InternedPath::from(root.join("shared.js"));
  let a = InternedPath::from(root.join("a.js"));
  let b = InternedPath::from(root.join("b.js"));
  let marker = InternedPath::from(root.join("marker.js"));
  let files = [
    (source.clone(), a.clone()),
    (source.clone(), b.clone()),
    (marker.clone(), marker.clone()),
  ];
  let identity = [(source.clone(), source.clone())];
  let watcher = manual_watcher();
  let (tx, mut notices) = unbounded_channel();
  let watch = |added, removed| {
    watch_subscriptions(
      &watcher,
      (added, removed),
      (&[], &[]),
      (&[], &[]),
      SystemTime::now(),
      Handler(tx.clone()),
      Handler(tx.clone()),
    )
  };

  write(&source, 0);
  write(&marker, 0);
  watch(&files, &[]).await;
  write(&source, 1);
  disk_event(&watcher, &source, FsEventKind::Change);
  expect_batch(&mut notices, &[&a, &b], &[], &[&a, &b], &[]).await;

  watch(&[], &files[..1]).await;
  write(&source, 2);
  watcher.trigger_event(&source, FsEventKind::Change);
  expect_batch(&mut notices, &[&b], &[], &[&b], &[]).await;

  watch(&identity, &files[1..2]).await;
  watch(&files[..1], &[]).await;
  write(&source, 3);
  disk_event(&watcher, &source, FsEventKind::Change);
  expect_batch(&mut notices, &[&source, &a], &[], &[&source, &a], &[]).await;

  watch(&[], &identity).await;
  write(&source, 4);
  disk_event(&watcher, &source, FsEventKind::Change);
  expect_batch(&mut notices, &[&a], &[], &[&a], &[]).await;

  watch(&[], &files[..1]).await;
  write(&source, 5);
  write(&marker, 1);
  disk_event(&watcher, &source, FsEventKind::Change);
  disk_event(&watcher, &marker, FsEventKind::Change);
  expect_batch(&mut notices, &[&marker], &[], &[&marker], &[]).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn directory_and_missing_subscribers_keep_their_own_paths() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let dir = InternedPath::from(root.join("src"));
  let child = InternedPath::from(dir.join("later.js"));
  let a = InternedPath::from(root.join("directory-a"));
  let b = InternedPath::from(root.join("directory-b"));
  let missing = InternedPath::from(root.join("requested-file.js"));
  fs::create_dir(&dir).expect("create subscribed directory");
  let watcher = manual_watcher();
  let (tx, mut notices) = unbounded_channel();
  watch_subscriptions(
    &watcher,
    (&[], &[]),
    (&[(dir.clone(), a.clone()), (dir.clone(), b.clone())], &[]),
    (&[(child.clone(), missing.clone())], &[]),
    SystemTime::now() + Duration::from_secs(60),
    Handler(tx.clone()),
    Handler(tx),
  )
  .await;

  disk_event(&watcher, &dir, FsEventKind::Change);
  expect_batch(&mut notices, &[&a, &b], &[], &[&a, &b], &[]).await;

  write(&child, 0);
  disk_event(&watcher, &child, FsEventKind::Create);
  expect_batch(
    &mut notices,
    &[&a, &b, &missing],
    &[],
    &[&child, &child, &missing],
    &[],
  )
  .await;
  fs::remove_file(&child).expect("delete subscribed file");
  disk_event(&watcher, &child, FsEventKind::Remove);
  expect_batch(
    &mut notices,
    &[&a, &b],
    &[&missing],
    &[&child, &child],
    &[&missing],
  )
  .await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn startup_scan_uses_the_same_file_subscriptions() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let source = InternedPath::from(root.join("shared.js"));
  let a = InternedPath::from(root.join("a.js"));
  let b = InternedPath::from(root.join("b.js"));
  write(&source, 0);
  let watcher = manual_watcher();
  let (tx, mut notices) = unbounded_channel();
  watch_subscriptions(
    &watcher,
    (&[(source.clone(), a.clone()), (source, b.clone())], &[]),
    (&[], &[]),
    (&[], &[]),
    SystemTime::UNIX_EPOCH,
    Handler(tx.clone()),
    Handler(tx),
  )
  .await;
  expect_batch(&mut notices, &[&a, &b], &[], &[&a, &b], &[]).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn queued_notification_keeps_its_file_and_directory_recipients() {
  let temp = tempfile::tempdir().expect("create fixture");
  let outer = InternedPath::from(temp.path().canonicalize().expect("resolve fixture"));
  let inner = InternedPath::from(outer.join("src"));
  let file = InternedPath::from(inner.join("a.js"));
  fs::create_dir(&inner).expect("create nested directory");
  write(&file, 0);
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![file.clone()], vec![]),
      (vec![outer.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register original recipients");
  analyzer.on_scanned_change(&file);
  tokio::time::timeout(Duration::from_secs(5), async {
    while !analyzer.has_queued_event() {
      tokio::time::sleep(Duration::from_millis(1)).await;
    }
  })
  .await
  .expect("notification queued before re-registration");
  analyzer
    .update(
      (vec![], vec![file.clone()]),
      (vec![inner], vec![outer.clone()]),
      (vec![], vec![]),
    )
    .await
    .expect("replace recipients");
  let mut queued = analyzer.next_events().await;
  queued.sort_by(|a, b| a.0.cmp(&b.0));
  assert_eq!(
    queued,
    vec![(outer, FsEventKind::Change), (file, FsEventKind::Change)]
  );
}
