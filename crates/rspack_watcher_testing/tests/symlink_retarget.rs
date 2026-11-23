#![cfg(target_os = "linux")]

use std::{fs, time::Duration};

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

fn empty() -> (
  std::iter::Empty<InternedPath>,
  std::iter::Empty<InternedPath>,
) {
  (std::iter::empty(), std::iter::empty())
}

async fn assert_target_creation_is_reported(retarget: bool, shared_modes: bool) {
  let fixture = TempDir::new().expect("create fixture");
  let base = fixture.path().canonicalize().expect("resolve fixture");
  let project = base.join("project");
  let a = base.join("a");
  let b = base.join("b");
  fs::create_dir(&project).expect("create project");
  for target in [&a, &b] {
    fs::create_dir_all(target.join("sub")).expect("create target subdirectory");
  }
  let link = project.join("link");
  std::os::unix::fs::symlink(&a, &link).expect("create symlink");
  let missing = InternedPath::from(link.join("sub/a.js"));
  let mut paths = vec![missing.clone()];
  if shared_modes {
    paths.push(InternedPath::from(link.join("sub/nested/b.js")));
  }
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: true,
      aggregate_timeout: Some(10),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut events) = unbounded_channel();
  watcher
    .watch(
      empty(),
      empty(),
      (paths.into_iter(), std::iter::empty()),
      std::time::SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(IgnoreIndividualEvents),
    )
    .await;
  // Let the initial scan of the missing dependencies finish before creating them.
  tokio::time::sleep(Duration::from_millis(100)).await;

  if retarget {
    let next = project.join("next-link");
    std::os::unix::fs::symlink(&b, &next).expect("create replacement symlink");
    fs::rename(next, &link).expect("replace symlink atomically");
  }
  watcher
    .watch(
      empty(),
      empty(),
      empty(),
      std::time::SystemTime::now(),
      Box::new(Changes(tx)),
      Box::new(IgnoreIndividualEvents),
    )
    .await;

  // Discard retargeting events so only the subsequent creation can satisfy the assertion.
  tokio::time::sleep(Duration::from_millis(100)).await;
  while events.try_recv().is_ok() {}
  assert!(!missing.exists());
  let target = if retarget { &b } else { &a };
  fs::write(target.join("sub/a.js"), "created after watch").expect("create dependency");
  let reported = tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(changed) = events.recv().await {
      if changed.contains(missing.to_str().expect("UTF-8 fixture path")) {
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
    "the current target must report creation"
  );
}

#[tokio::test]
async fn test_unchanged_symlink_target_reports_creation() {
  assert_target_creation_is_reported(false, false).await;
}

#[tokio::test]
async fn test_same_shape_symlink_retarget_reports_creation() {
  assert_target_creation_is_reported(true, false).await;
}

#[tokio::test]
async fn test_same_shape_symlink_retarget_with_shared_watch_directory_reports_creation() {
  assert_target_creation_is_reported(true, true).await;
}
