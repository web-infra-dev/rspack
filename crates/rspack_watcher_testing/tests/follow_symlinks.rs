use std::{
  fs,
  path::{Path, PathBuf},
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions, testing::DirectoryAnalyzer,
};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

struct Changes(UnboundedSender<(FxHashSet<String>, FxHashSet<String>)>);

impl EventAggregateHandler for Changes {
  fn on_event_handle(&self, changed: FxHashSet<String>, removed: FxHashSet<String>) {
    let _ = self.0.send((changed, removed));
  }
}

struct IgnoreIndividualEvents;

impl EventHandler for IgnoreIndividualEvents {}

fn empty() -> (
  std::iter::Empty<InternedPath>,
  std::iter::Empty<InternedPath>,
) {
  (std::iter::empty(), std::iter::empty())
}

fn create_file_link(target: &Path, link: &Path) {
  #[cfg(unix)]
  std::os::unix::fs::symlink(target, link).expect("create unix file symlink");
  #[cfg(windows)]
  std::os::windows::fs::symlink_file(target, link).expect("create windows file symlink");
}

fn create_dir_link(target: &Path, link: &Path) {
  #[cfg(unix)]
  std::os::unix::fs::symlink(target, link).expect("create unix dir symlink");
  #[cfg(windows)]
  std::os::windows::fs::symlink_dir(target, link).expect("create windows dir symlink");
}

fn set_modified(path: &Path, mtime: SystemTime) {
  fs::File::options()
    .write(true)
    .open(path)
    .expect("open file for set_modified")
    .set_modified(mtime)
    .expect("set mtime");
}

#[tokio::test]
async fn unregistering_symlink_dependencies_leaves_zero_index_entries() {
  let fixture = TempDir::new().expect("create fixture");
  let base: PathBuf = fixture.path().canonicalize().expect("resolve fixture");
  let app = base.join("app");
  let ext_dir = base.join("ext_dir");
  let ext_file = base.join("ext_file.js");
  fs::create_dir_all(ext_dir.join("nested")).expect("create external dir");
  fs::create_dir_all(&app).expect("create app dir");
  fs::write(&ext_file, "export default 1;\n").expect("write external file");
  fs::write(ext_dir.join("nested/mod.js"), "export const x = 1;\n").expect("write nested file");

  let file_link = InternedPath::from(app.join("file_link.js"));
  let dir_link = InternedPath::from(app.join("dir_link"));
  let missing_via_link = InternedPath::from(app.join("unlinked_pkg/index.js"));
  create_file_link(&ext_file, file_link.as_ref());
  create_dir_link(&ext_dir, dir_link.as_ref());

  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: true,
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, _rx) = unbounded_channel();

  watcher
    .watch(
      (vec![file_link.clone()].into_iter(), std::iter::empty()),
      (vec![dir_link.clone()].into_iter(), std::iter::empty()),
      (
        vec![missing_via_link.clone()].into_iter(),
        std::iter::empty(),
      ),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;

  let (recorded_before, registered_before) = watcher.index_sizes();
  assert!(
    recorded_before > 0,
    "expected recorded entries while watching"
  );
  assert!(
    registered_before > 0,
    "expected registered ancestor entries while watching"
  );

  watcher
    .watch(
      (std::iter::empty(), vec![file_link].into_iter()),
      (std::iter::empty(), vec![dir_link].into_iter()),
      (std::iter::empty(), vec![missing_via_link].into_iter()),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;

  assert_eq!(
    watcher.index_sizes(),
    (0, 0),
    "all recorded and link-mapping indices must return to (0, 0) after unregistering"
  );
  watcher.close().await.expect("close watcher");
}

#[cfg(unix)]
#[tokio::test]
async fn follow_symlinks_false_detects_atomic_symlink_swap_between_aged_targets() {
  let fixture = TempDir::new().expect("create fixture");
  let base = fixture.path().canonicalize().expect("resolve fixture");
  let old = SystemTime::now() - Duration::from_secs(3600);
  let target_a = base.join("a.js");
  let target_b = base.join("b.js");
  fs::write(&target_a, "export default 'a';\n").expect("write a");
  fs::write(&target_b, "export default 'b';\n").expect("write b");
  // Same aged mtime on both targets (#16701): only `read_link` / link state distinguishes them.
  set_modified(&target_a, old);
  set_modified(&target_b, old);

  let link = InternedPath::from(base.join("current.js"));
  create_file_link(&target_a, link.as_ref());

  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: false,
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    rspack_watcher::FsWatcherIgnored::Paths(vec!["**/packages/my-pkg/**".to_string()]),
  );
  let (tx, mut events) = unbounded_channel();
  watcher
    .watch(
      (vec![link.clone()].into_iter(), std::iter::empty()),
      empty(),
      empty(),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;

  tokio::time::sleep(Duration::from_millis(250)).await;
  while events.try_recv().is_ok() {}

  let staging = base.join("staging.js");
  create_file_link(&target_b, &staging);
  fs::rename(&staging, link.as_ref()).expect("atomically swap symlink");

  let link_str = link.to_string_lossy().into_owned();
  let reported = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some((changed, _)) = events.recv().await {
      if changed.contains(&link_str) {
        return true;
      }
    }
    false
  })
  .await;

  watcher.close().await.expect("close watcher");
  assert_eq!(
    reported,
    Ok(true),
    "atomic symlink swap between equal-mtime targets must report a change even when follow_symlinks is false"
  );
}

#[cfg(unix)]
#[tokio::test]
async fn same_shape_symlink_retarget_with_shared_watch_directory_reports_creation_cross_unix() {
  let fixture = TempDir::new().expect("create fixture");
  let base = fixture.path().canonicalize().expect("resolve fixture");
  let project = base.join("project");
  let a = base.join("a");
  let b = base.join("b");
  fs::create_dir(&project).expect("create project");
  for target in [&a, &b] {
    fs::create_dir_all(target.join("sub")).expect("create target sub");
  }
  let link = project.join("link");
  create_dir_link(&a, &link);
  let missing_a = InternedPath::from(link.join("sub/a.js"));
  let missing_b = InternedPath::from(link.join("sub/nested/b.js"));

  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: true,
      aggregate_timeout: Some(15),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut events) = unbounded_channel();
  watcher
    .watch(
      empty(),
      empty(),
      (
        vec![missing_a.clone(), missing_b].into_iter(),
        std::iter::empty(),
      ),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;

  let next = project.join("next-link");
  create_dir_link(&b, &next);
  fs::rename(&next, &link).expect("retarget symlink");

  watcher
    .watch(
      empty(),
      empty(),
      empty(),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;
  while events.try_recv().is_ok() {}

  fs::write(b.join("sub/a.js"), "created").expect("create target file");
  let missing_str = missing_a.to_string_lossy().into_owned();
  let reported = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some((changed, _)) = events.recv().await {
      if changed.contains(&missing_str) {
        return true;
      }
    }
    false
  })
  .await;

  watcher.close().await.expect("close watcher");
  assert_eq!(reported, Ok(true));
}

#[tokio::test]
async fn directory_analyzer_preserves_explicit_parent_root_and_splits_disjoint_trees() {
  let fixture_a = TempDir::new().expect("create fixture a");
  let fixture_b = TempDir::new().expect("create fixture b");
  let root_a = fixture_a.path().canonicalize().expect("resolve a");
  let root_b = fixture_b.path().canonicalize().expect("resolve b");
  fs::create_dir_all(root_a.join("only_child/deep")).expect("create deep dir");
  let deep_file = InternedPath::from(root_a.join("only_child/deep/entry.js"));
  let other_file = InternedPath::from(root_b.join("pkg/index.js"));
  fs::create_dir_all(root_b.join("pkg")).expect("create b pkg");
  fs::write(&deep_file, "1").expect("write deep file");
  fs::write(&other_file, "2").expect("write other file");

  let explicit_root = InternedPath::from(root_a.clone());
  let analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![deep_file, other_file], vec![]),
      (vec![explicit_root.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("update analyzer");

  let patterns = analyzer.analyze();
  assert!(
    patterns.iter().any(|(path, _)| *path == explicit_root),
    "explicitly watched parent context must remain in analyzed patterns: {patterns:?}"
  );
  assert!(
    patterns
      .iter()
      .any(|(path, _)| path.as_ref().starts_with(&root_b)),
    "external tree must have its own watch pattern: {patterns:?}"
  );
}

#[tokio::test]
async fn watch_mode_transition_recursive_to_non_recursive_and_back_delivers_events() {
  let fixture = TempDir::new().expect("create fixture");
  let base = fixture.path().canonicalize().expect("resolve fixture");
  let root = base.join("workspace");
  fs::create_dir_all(root.join("existing_sub")).expect("create existing_sub");
  let existing_file = InternedPath::from(root.join("existing_sub/keep.js"));
  fs::write(&existing_file, "export const v = 1;\n").expect("write existing file");
  let missing_deep = InternedPath::from(root.join("missing_branch/nested/target.js"));

  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: true,
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut events) = unbounded_channel();

  // Round 1: `missing_deep` forces `root` to be watched `Recursive` while `root/existing_sub` is `NonRecursive`.
  watcher
    .watch(
      (vec![existing_file.clone()].into_iter(), std::iter::empty()),
      empty(),
      (vec![missing_deep.clone()].into_iter(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;

  // Round 2: remove `missing_deep` and watch `root` as context (`NonRecursive` on Linux/testing).
  let root_ctx = InternedPath::from(root.clone());
  watcher
    .watch(
      empty(),
      (vec![root_ctx.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), vec![missing_deep.clone()].into_iter()),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;

  // Round 3: unregister `root_ctx` and re-add `missing_deep` (`Recursive` on `root` again).
  watcher
    .watch(
      empty(),
      (std::iter::empty(), vec![root_ctx].into_iter()),
      (vec![missing_deep.clone()].into_iter(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;
  while events.try_recv().is_ok() {}

  // Both `existing_file` and newly created `missing_deep` must still emit events!
  fs::write(&existing_file, "export const v = 2;\n").expect("modify existing_file");
  fs::create_dir_all(root.join("missing_branch/nested")).expect("create missing_branch");
  fs::write(&missing_deep, "export default 42;\n").expect("create missing_deep");

  let existing_str = existing_file.to_string_lossy().into_owned();
  let missing_str = missing_deep.to_string_lossy().into_owned();
  let mut seen = FxHashSet::default();
  let _ = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some((changed, _)) = events.recv().await {
      seen.extend(changed);
      if seen.contains(&existing_str) && seen.contains(&missing_str) {
        break;
      }
    }
  })
  .await;

  watcher.close().await.expect("close watcher");
  assert!(
    seen.contains(&existing_str),
    "existing child directory watch must survive Recursive -> NonRecursive -> Recursive transitions: {seen:?}"
  );
  assert!(
    seen.contains(&missing_str),
    "re-promoted Recursive missing ancestor watch must observe deep creation: {seen:?}"
  );
}

#[tokio::test]
async fn follow_symlinks_false_observes_files_and_missing_under_symlinked_parent_dir() {
  let fixture = TempDir::new().expect("create fixture");
  let base = fixture.path().canonicalize().expect("resolve fixture");
  let packages_pkg = base.join("packages/my-pkg/dist");
  let node_modules = base.join("app/node_modules");
  fs::create_dir_all(&packages_pkg).expect("create packages/my-pkg/dist");
  fs::create_dir_all(&node_modules).expect("create app/node_modules");
  fs::write(packages_pkg.join("index.js"), "export default 1;\n").expect("write index.js");

  let symlinked_pkg = node_modules.join("my-pkg");
  create_dir_link(&base.join("packages/my-pkg"), &symlinked_pkg);

  let logical_file = InternedPath::from(symlinked_pkg.join("dist/index.js"));
  let logical_missing = InternedPath::from(symlinked_pkg.join("dist/extra/lazy.js"));
  let missing_ancestor_probe = InternedPath::from(base.join("packages/unborn/deep/probe.js"));

  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: false,
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    rspack_watcher::FsWatcherIgnored::Paths(vec!["**/packages/my-pkg/**".to_string()]),
  );
  let (tx, mut events) = unbounded_channel();

  // Round 1: also register `missing_ancestor_probe` so `base/packages` is watched `Recursive` alongside `packages/my-pkg/dist` (`NonRecursive`).
  watcher
    .watch(
      (vec![logical_file.clone()].into_iter(), std::iter::empty()),
      empty(),
      (
        vec![logical_missing.clone(), missing_ancestor_probe.clone()].into_iter(),
        std::iter::empty(),
      ),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;

  // Round 2: drop `missing_ancestor_probe` completely (`Recursive` ancestor disappears from `next`, not downgraded!).
  watcher
    .watch(
      empty(),
      empty(),
      (std::iter::empty(), vec![missing_ancestor_probe].into_iter()),
      SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(150)).await;
  while events.try_recv().is_ok() {}

  fs::write(packages_pkg.join("index.js"), "export default 2;\n").expect("update index.js");
  fs::create_dir_all(packages_pkg.join("extra")).expect("create extra dir");
  fs::write(packages_pkg.join("extra/lazy.js"), "export default 3;\n").expect("write lazy.js");

  let file_str = logical_file.to_string_lossy().into_owned();
  let missing_str = logical_missing.to_string_lossy().into_owned();
  let mut seen = FxHashSet::default();
  let _ = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some((changed, _)) = events.recv().await {
      seen.extend(changed);
      if seen.contains(&file_str) && seen.contains(&missing_str) {
        break;
      }
    }
  })
  .await;

  watcher.close().await.expect("close watcher");
  assert!(
    seen.contains(&file_str),
    "file under symlinked parent must report changes when follow_symlinks is false: {seen:?}"
  );
  assert!(
    seen.contains(&missing_str),
    "missing file under symlinked parent must report creation when follow_symlinks is false: {seen:?}"
  );
}
