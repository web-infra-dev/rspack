use std::{
  fs,
  path::{Path, PathBuf},
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions, TimeInfoEntry,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

struct Changes(UnboundedSender<String>);

impl EventHandler for Changes {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send(path);
    Ok(())
  }
}

impl EventAggregateHandler for Changes {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}

  fn on_error(&self, error: rspack_error::Error) {
    panic!("watch registration failed: {error}");
  }
}

async fn watch(
  watcher: &FsWatcher,
  contexts: &[&Path],
  added_missing: &[&Path],
  removed_missing: &[&Path],
) -> UnboundedReceiver<String> {
  let (tx, rx) = unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (
        contexts.iter().copied().map(InternedPath::from),
        std::iter::empty(),
      ),
      (
        added_missing.iter().copied().map(InternedPath::from),
        removed_missing.iter().copied().map(InternedPath::from),
      ),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(Changes(tx)),
    )
    .await;
  rx
}

async fn wait_for(rx: &mut UnboundedReceiver<String>, path: &Path) {
  tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(changed) = rx.recv().await {
      if Path::new(&changed) == path {
        return;
      }
    }
    panic!("event channel closed before {}", path.display());
  })
  .await
  .unwrap_or_else(|_| panic!("no disk event for {}", path.display()));
}

fn fixture() -> (tempfile::TempDir, PathBuf, FsWatcher) {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  (temp, root, watcher)
}

#[tokio::test]
async fn recursive_watches_stay_within_the_requested_trees() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let left = InternedPath::from(root.join("left"));
  let right = InternedPath::from(root.join("right"));
  fs::create_dir(&left).expect("create left tree");
  fs::create_dir(&right).expect("create right tree");
  let child = InternedPath::from(left.join("child.js"));
  let other = InternedPath::from(right.join("other.js"));
  let analyzer = rspack_watcher::testing::DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![child, other], vec![]),
      (vec![left.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register both trees");
  assert_eq!(
    analyzer
      .recursive_roots()
      .into_iter()
      .collect::<FxHashSet<_>>(),
    [left.clone(), right.clone()]
      .into_iter()
      .collect::<FxHashSet<_>>()
  );
  analyzer
    .update(
      (vec![], vec![]),
      (vec![], vec![left.clone()]),
      (vec![], vec![]),
    )
    .await
    .expect("remove only the left context");
  assert_eq!(
    analyzer
      .recursive_roots()
      .into_iter()
      .collect::<FxHashSet<_>>(),
    [left, right].into_iter().collect::<FxHashSet<_>>()
  );
}

#[tokio::test]
async fn disjoint_roots_and_shared_directories_keep_their_surviving_subscriptions() {
  let (_temp, root, watcher) = fixture();
  let left = root.join("left");
  let right = root.join("right");
  fs::create_dir(&left).expect("create first tree");
  fs::create_dir(&right).expect("create second tree");
  let first = left.join("first.js");
  let second = left.join("second.js");
  let other = right.join("other.js");
  let mut events = watch(&watcher, &[], &[&first, &second, &other], &[]).await;
  fs::write(&first, "first").expect("write first tree");
  wait_for(&mut events, &first).await;
  fs::write(&other, "other").expect("write second tree");
  wait_for(&mut events, &other).await;

  let mut events = watch(&watcher, &[], &[], &[&first, &other]).await;
  fs::write(&second, "second").expect("write surviving subscription");
  wait_for(&mut events, &second).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn removing_a_recursive_root_keeps_a_registered_child_listening() {
  let (_temp, root, watcher) = fixture();
  let kept = root.join("kept");
  fs::create_dir(&kept).expect("create registered child's directory");
  let child = kept.join("child.js");
  let deep = root.join("missing/deep.js");
  let _events = watch(&watcher, &[], &[&child, &deep], &[]).await;

  let mut events = watch(&watcher, &[], &[], &[&deep]).await;
  fs::write(&child, "child").expect("write registered child");
  wait_for(&mut events, &child).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn a_recovered_context_observes_its_contents_and_later_writes() {
  let (_temp, root, watcher) = fixture();
  let context = root.join("context");
  let staged = root.join("staged");
  fs::create_dir(&staged).expect("create staged directory");
  fs::write(staged.join("before.js"), "before").expect("populate staged directory");
  let mut events = watch(&watcher, &[&context], &[], &[]).await;

  fs::rename(staged, &context).expect("restore context");
  wait_for(&mut events, &context).await;
  let before = context.join("before.js");
  let (files, _) = watcher.collect_time_info_entries();
  assert!(
    files.iter().any(
      |(name, entry)| Path::new(name) == before && matches!(entry, TimeInfoEntry::Entry { .. })
    ),
    "recovery must scan the moved-in contents: {files:?}"
  );
  let after = context.join("after.js");
  fs::write(&after, "after").expect("write after recovery scan");
  wait_for(&mut events, &after).await;
  watcher.close().await.expect("close watcher");
}
