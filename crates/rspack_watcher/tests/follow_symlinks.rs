use std::{
  fs,
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

struct Changes(UnboundedSender<FxHashSet<String>>);

impl EventAggregateHandler for Changes {
  fn on_event_handle(&self, changed: FxHashSet<String>, _removed: FxHashSet<String>) {
    let _ = self.0.send(changed);
  }
}

struct IgnoreIndividualEvents;

impl EventHandler for IgnoreIndividualEvents {}

#[tokio::test]
async fn root_analyzer_preserves_explicit_parent_context_with_single_deep_child() {
  let fixture_a = TempDir::new().expect("create fixture a");
  let fixture_b = TempDir::new().expect("create fixture b");
  let root_a = fixture_a.path().canonicalize().expect("resolve a");
  let root_b = fixture_b.path().canonicalize().expect("resolve b");
  fs::create_dir_all(root_a.join("only_child/deep")).expect("create deep child");
  fs::create_dir_all(root_b.join("external_pkg")).expect("create external pkg");
  let deep_file = InternedPath::from(root_a.join("only_child/deep/entry.js"));
  let ext_file = InternedPath::from(root_b.join("external_pkg/index.js"));
  fs::write(&deep_file, "1").expect("write deep file");
  fs::write(&ext_file, "2").expect("write external file");

  let explicit_context = InternedPath::from(root_a.clone());
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: true,
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut events) = unbounded_channel();
  watcher
    .watch(
      (
        vec![deep_file, ext_file.clone()].into_iter(),
        std::iter::empty(),
      ),
      (
        vec![explicit_context.clone()].into_iter(),
        std::iter::empty(),
      ),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;

  tokio::time::sleep(Duration::from_millis(250)).await;
  while events.try_recv().is_ok() {}

  // Creating a sibling at the top of `explicit_context` proves `WatcherRootAnalyzer`
  // did not narrow `explicit_context` down to `only_child/deep`.
  fs::write(root_a.join("new_top_level.js"), "top").expect("write top level file");
  let ctx_str = explicit_context.to_string_lossy().into_owned();
  let got_ctx = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(changed) = events.recv().await {
      if changed.contains(&ctx_str) {
        return true;
      }
    }
    false
  })
  .await;
  assert_eq!(
    got_ctx,
    Ok(true),
    "explicit parent context must observe top-level additions"
  );

  // Editing `ext_file` in disjoint tree `root_b` proves multi-root watching is active.
  while events.try_recv().is_ok() {}
  fs::write(&ext_file, "updated").expect("update external file");
  let ext_str = ext_file.to_string_lossy().into_owned();
  let got_ext = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(changed) = events.recv().await {
      if changed.contains(&ext_str) {
        return true;
      }
    }
    false
  })
  .await;
  watcher.close().await.expect("close watcher");
  assert_eq!(
    got_ext,
    Ok(true),
    "disjoint external root must observe file updates"
  );
}

#[tokio::test]
async fn root_analyzer_keeps_deep_child_when_parent_context_or_cross_set_path_removed() {
  let fixture = TempDir::new().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve root");
  fs::create_dir_all(root.join("pkg/sub/deep")).expect("create deep dir");
  let deep_file = InternedPath::from(root.join("pkg/sub/deep/index.js"));
  let shared_file = InternedPath::from(root.join("pkg/shared.js"));
  fs::write(&deep_file, "export default 1;\n").expect("write deep file");
  fs::write(&shared_file, "export default 2;\n").expect("write shared file");

  let parent_ctx = InternedPath::from(root.join("pkg"));
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: true,
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut events) = unbounded_channel();

  // Round 1: register parent_ctx as directory, deep_file as file, and shared_file in both files + missing.
  watcher
    .watch(
      (
        vec![deep_file.clone(), shared_file.clone()].into_iter(),
        std::iter::empty(),
      ),
      (vec![parent_ctx.clone()].into_iter(), std::iter::empty()),
      (vec![shared_file.clone()].into_iter(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  while events.try_recv().is_ok() {}

  // Round 2: unregister `parent_ctx` from directories and `shared_file` from missing only.
  // Both `deep_file` and `shared_file` must remain watched via `files`.
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![parent_ctx].into_iter()),
      (std::iter::empty(), vec![shared_file.clone()].into_iter()),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  while events.try_recv().is_ok() {}

  fs::write(&deep_file, "export default 10;\n").expect("update deep file");
  fs::write(&shared_file, "export default 20;\n").expect("update shared file");

  let deep_str = deep_file.to_string_lossy().into_owned();
  let shared_str = shared_file.to_string_lossy().into_owned();
  let mut seen = FxHashSet::default();
  let _ = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(changed) = events.recv().await {
      seen.extend(changed);
      if seen.contains(&deep_str) && seen.contains(&shared_str) {
        break;
      }
    }
  })
  .await;

  watcher.close().await.expect("close watcher");
  assert!(
    seen.contains(&deep_str),
    "deep_file must remain watched after unregistering parent context: {seen:?}"
  );
  assert!(
    seen.contains(&shared_str),
    "shared_file must remain watched in files after removing from missing: {seen:?}"
  );
}
