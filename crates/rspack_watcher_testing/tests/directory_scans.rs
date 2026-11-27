#[path = "scan_support/pause.rs"]
mod pause;

use std::{
  fs,
  path::Path,
  time::{Duration, SystemTime},
};

use pause::pause_at as block_first_visit;
use rspack_watcher::{TimeInfoEntry, testing::DirectoryAnalyzer};
use tokio::time::timeout;

#[tokio::test]
async fn targeted_scans_wait_for_their_own_directory_only() {
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let busy = root.join("busy");
  fs::create_dir(&busy).expect("create busy directory");
  let blocker = busy.join("blocker");
  fs::write(&blocker, "").expect("write fixture");
  let queued = busy.join("queued");
  let other = root.join("other");
  let (ignored, waiting, release) = block_first_visit(&blocker);
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      Default::default(),
      (vec![busy.into()], vec![]),
      Default::default(),
    )
    .await
    .expect("update watch dependencies");
  let scan = analyzer.scan_contexts();
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("scan reached the fixture before timeout")
    .expect("gate signaled");
  for file in [&queued, &other] {
    fs::write(file, "").expect("write fixture");
  }
  analyzer
    .update(
      Default::default(),
      Default::default(),
      (vec![(&queued).into(), (&other).into()], vec![]),
    )
    .await
    .expect("update watch dependencies");
  analyzer.scan(SystemTime::now() + Duration::from_secs(60));
  let has_time = |path: &Path| {
    analyzer
      .collect_time_info_entries()
      .0
      .iter()
      .any(|(name, value)| Path::new(name) == path && matches!(value, TimeInfoEntry::Entry { .. }))
  };
  timeout(Duration::from_secs(5), async {
    while !has_time(&other) {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("an independent directory can finish");
  let ran_early = has_time(&queued);
  release.send(()).expect("release scan");
  scan.await.expect("finish directory scan");
  assert!(
    !ran_early,
    "the targeted scan overtook its directory enumeration"
  );
  timeout(Duration::from_secs(5), async {
    while !has_time(&queued) {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("the queued scan completes");
}

#[tokio::test]
async fn a_blocked_context_scan_does_not_prevent_unwatch_or_close() {
  use rspack_util::fx_hash::FxHashSet;
  use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher};
  struct Handler;
  impl EventHandler for Handler {}
  impl EventAggregateHandler for Handler {
    fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}
  }
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let child = root.join("child");
  fs::write(&child, "").expect("write fixture");
  let (ignored, waiting, release) = block_first_visit(&child);
  let watcher = FsWatcher::new(Default::default(), ignored);
  let watch = |add: bool| {
    watcher.watch(
      (std::iter::empty(), std::iter::empty()),
      (
        add.then(|| (&root).into()).into_iter(),
        (!add).then(|| (&root).into()).into_iter(),
      ),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now() + Duration::from_secs(60),
      Box::new(Handler),
      Box::new(Handler),
    )
  };
  let scanning = watch(true);
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("scan reached the fixture before timeout")
    .expect("gate signaled");
  let retired = timeout(Duration::from_secs(1), watch(false)).await;
  let closed = timeout(Duration::from_secs(1), watcher.close()).await;
  let _ = release.send(());
  scanning.await;
  retired.expect("unsubscribe while the old scan is still waiting");
  closed
    .expect("close while the old scan is still waiting")
    .expect("close watcher");
}
