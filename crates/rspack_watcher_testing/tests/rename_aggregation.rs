use std::{
  fs,
  path::Path,
  time::{Duration, SystemTime},
};

use rspack_fs::{AbsPath, AbsPathSet};
use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

struct Aggregate(UnboundedSender<(AbsPathSet, AbsPathSet)>);

impl EventAggregateHandler for Aggregate {
  fn on_event_handle(&self, changed: AbsPathSet, removed: AbsPathSet) {
    let _ = self.0.send((changed, removed));
  }
}

struct Raw(UnboundedSender<AbsPath>);
impl EventHandler for Raw {
  fn on_delete(&self, path: AbsPath) -> rspack_error::Result<()> {
    let _ = self.0.send(path);
    Ok(())
  }
}

fn write_old(path: &Path, content: &str) {
  fs::write(path, content).expect("write fixture");
  fs::File::options()
    .write(true)
    .open(path)
    .expect("open fixture")
    .set_modified(SystemTime::now() - Duration::from_secs(60))
    .expect("set old mtime");
}

async fn assert_rename_result(operation: &str) {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let file = base.join("index.js");
  let backup = base.join("backup.js");
  let replacement = base.join("replacement.js");
  write_old(&file, "old");
  fs::write(&replacement, "new").expect("write replacement");
  let mtime = fs::metadata(&file)
    .expect("stat file")
    .modified()
    .expect("read mtime");
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(500),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut rx) = unbounded_channel();
  let (raw_tx, mut raw_rx) = unbounded_channel();
  let file_abs = AbsPath::try_from_std_path(&file).expect("valid utf-8 path");
  watcher
    .watch(
      (vec![file_abs.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Aggregate(tx)),
      Box::new(Raw(raw_tx)),
    )
    .await;
  fs::rename(&file, &backup).expect("rename source away");
  let removal = tokio::time::timeout(Duration::from_secs(5), raw_rx.recv()).await;
  if removal.is_err() {
    watcher.close().await.expect("close watcher");
  }
  assert_eq!(
    removal.expect("watcher should observe the rename source removal"),
    Some(file_abs.clone())
  );
  match operation {
    "replace" => fs::rename(&replacement, &file).expect("install replacement"),
    "round-trip" => {
      fs::rename(&backup, &file).expect("restore source");
      assert_eq!(
        fs::metadata(&file)
          .expect("stat restored file")
          .modified()
          .expect("read restored mtime"),
        mtime
      );
    }
    "remove" => {}
    _ => unreachable!(),
  }
  let batch = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;
  watcher.close().await.expect("close watcher");
  let (changed, removed) = batch
    .expect("rename should produce an aggregated batch")
    .expect("aggregate handler should remain active");
  if operation == "remove" {
    assert!(
      removed.contains(&file_abs),
      "rename source was not removed: {removed:?}"
    );
    assert!(
      !changed.contains(&file_abs),
      "rename source was reported changed: {changed:?}"
    );
  } else {
    assert!(
      changed.contains(&file_abs),
      "existing replacement was not changed: {changed:?}"
    );
    assert!(
      !removed.contains(&file_abs),
      "existing replacement was reported removed: {removed:?}"
    );
  }
}

#[tokio::test]
async fn test_file_rename_reports_source_as_removed() {
  assert_rename_result("remove").await;
}

#[tokio::test]
async fn test_editor_replacement_is_changed_only() {
  assert_rename_result("replace").await;
}

#[tokio::test]
async fn test_rename_round_trip_with_unchanged_mtime_is_changed_only() {
  assert_rename_result("round-trip").await;
}
