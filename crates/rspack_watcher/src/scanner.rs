#[cfg(any(test, feature = "testing"))]
use std::ops::Deref;
use std::{sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathDashSet};
use rspack_util::time::system_time_to_millis;
use tokio::sync::mpsc::UnboundedSender;

use super::{EventBatch, FsEvent, FsEventKind, PathManager};
use crate::{
  paths::{ScanPlan, disk_mtime, links::DepKind},
  time::{mtime_accuracy, mtime_safe_time},
};

pub struct Scanner {
  path_manager: Arc<PathManager>,
  tx: Option<UnboundedSender<EventBatch>>,
}

impl Scanner {
  pub fn new(tx: UnboundedSender<EventBatch>, path_manager: Arc<PathManager>) -> Self {
    Self {
      path_manager,
      tx: Some(tx),
    }
  }

  #[cfg(any(test, feature = "testing"))]
  pub fn scan(&self, start_time: SystemTime) {
    let accessor = self.path_manager.access();
    let (files, directories, missing_added) = match self.path_manager.take_scan_plan() {
      Some(plan) => (plan.files, plan.directories, plan.missing),
      None => (
        accessor
          .files()
          .1
          .iter()
          .map(|p| p.deref().clone())
          .collect(),
        accessor
          .directories()
          .1
          .iter()
          .map(|p| p.deref().clone())
          .collect(),
        accessor
          .missing()
          .1
          .iter()
          .map(|p| p.deref().clone())
          .collect(),
      ),
    };
    self.run_scan(start_time, files, directories, missing_added);
  }

  pub(crate) fn scan_with_plan(&self, start_time: SystemTime, plan: ScanPlan) {
    #[cfg(any(test, feature = "testing"))]
    let _ = self.path_manager.take_scan_plan();
    self.run_scan(start_time, plan.files, plan.directories, plan.missing);
  }

  fn run_scan(
    &self,
    start_time: SystemTime,
    files: Vec<InternedPath>,
    directories: Vec<InternedPath>,
    missing_added: Vec<InternedPath>,
  ) {
    if files.is_empty() && directories.is_empty() && missing_added.is_empty() {
      return;
    }

    let accessor = self.path_manager.access();
    if let Some(tx) = self.tx.clone() {
      let missing = accessor.missing().0.clone();
      let files_tx = tx.clone();
      let files_path_manager = Arc::clone(&self.path_manager);
      tokio::spawn(async move {
        let mut removed = absent_paths(&files, &missing, |path| {
          files_path_manager.is_dep_present_on_disk(DepKind::File, path)
        });
        removed.retain(|path| !files_path_manager.is_known_absent(path));
        let changed = files
          .iter()
          .filter(|p| dep_changed_since(&files_path_manager, DepKind::File, p, start_time))
          .cloned()
          .collect::<Vec<_>>();
        for path in &changed {
          if !files_path_manager.is_registered(DepKind::File, path) {
            continue;
          }
          let mtime = disk_mtime(path).or_else(|| symlink_mtime(path));
          if let Some(mtime) = mtime {
            files_path_manager.set_reported_file_time(path, mtime);
          }
        }
        _ = send_events(removed, FsEventKind::Remove, &files_tx);
        _ = send_events(changed, FsEventKind::Change, &files_tx);
      });

      let missing = accessor.missing().0.clone();
      let dirs_tx = tx.clone();
      let dirs_path_manager = Arc::clone(&self.path_manager);
      tokio::spawn(async move {
        _ = send_events(
          absent_paths(&directories, &missing, |path| {
            dirs_path_manager.is_dep_present_on_disk(DepKind::Directory, path)
          }),
          FsEventKind::Remove,
          &dirs_tx,
        );
        let changed = directories
          .into_iter()
          .filter(|p| dep_changed_since(&dirs_path_manager, DepKind::Directory, p, start_time))
          .collect::<Vec<_>>();
        _ = send_events(changed, FsEventKind::Change, &dirs_tx);
      });

      let path_manager = Arc::clone(&self.path_manager);
      tokio::spawn(async move {
        let mut created = Vec::new();
        for path in missing_added {
          if !path_manager.is_registered(DepKind::Missing, &path) {
            continue;
          }
          let bound = path_manager.bound_dep(DepKind::Missing, &path);
          let target_active = bound.as_ref().is_none_or(|b| b.target_active);
          let target_path = bound
            .as_ref()
            .map_or_else(|| path.clone(), |b| b.target.clone());
          let links_changed = bound
            .as_ref()
            .is_some_and(|b| b.links.iter().any(|l| symlink_changed_since(l, start_time)));

          if target_active && let Some(mtime) = disk_mtime(&target_path) {
            if !path_manager.is_registered(DepKind::Missing, &path) {
              continue;
            }
            if changed_since(&target_path, start_time) || links_changed {
              path_manager.set_reported_file_time(&path, mtime);
              created.push(path);
            } else {
              path_manager.set_registered_file_time(&path, mtime, true, true);
            }
          } else if links_changed && path_manager.is_registered(DepKind::Missing, &path) {
            created.push(path);
          }
        }
        _ = send_events(created, FsEventKind::Create, &tx);
      });
    }
  }

  pub fn close(&mut self) {
    self.tx.take();
  }
}

fn absent_paths(
  paths: &[InternedPath],
  missing: &InternedPathDashSet,
  present: impl Fn(&InternedPath) -> bool,
) -> Vec<InternedPath> {
  paths
    .iter()
    .filter(|path| !present(path) && !missing.contains(*path))
    .cloned()
    .collect()
}

fn send_events(
  paths: Vec<InternedPath>,
  kind: FsEventKind,
  tx: &UnboundedSender<EventBatch>,
) -> bool {
  if paths.is_empty() {
    return true;
  }
  let events = paths
    .into_iter()
    .map(|path| FsEvent { path, kind })
    .collect::<Vec<_>>();
  tx.send(events).is_ok()
}

fn symlink_mtime(path: &InternedPath) -> Option<SystemTime> {
  std::fs::symlink_metadata(path)
    .and_then(|m| m.modified().or_else(|_| m.created()))
    .ok()
}

fn meta_changed_since(meta: std::io::Result<std::fs::Metadata>, start_time: SystemTime) -> bool {
  let Ok(metadata) = meta else {
    return false;
  };
  let mtime = metadata.modified();
  if let Ok(mtime) = &mtime {
    mtime_accuracy(*mtime);
  }
  let Ok(mtime) = mtime.or_else(|_| metadata.created()) else {
    return false;
  };
  mtime_safe_time(mtime) >= system_time_to_millis(start_time)
}

fn symlink_changed_since(path: &InternedPath, start_time: SystemTime) -> bool {
  meta_changed_since(std::fs::symlink_metadata(path), start_time)
}

fn changed_since(path: &InternedPath, start_time: SystemTime) -> bool {
  meta_changed_since(path.metadata(), start_time)
}

fn dep_changed_since(
  path_manager: &PathManager,
  kind: DepKind,
  logical_path: &InternedPath,
  start_time: SystemTime,
) -> bool {
  let leaf_changed = |target: &InternedPath| {
    if kind == DepKind::File && !path_manager.follows_symlinks() && target.is_symlink() {
      symlink_changed_since(target, start_time)
    } else {
      changed_since(target, start_time)
    }
  };
  if let Some(bound) = path_manager.bound_dep(kind, logical_path) {
    let target_changed = bound.target_active && leaf_changed(&bound.target);
    let link_changed = bound
      .links
      .iter()
      .any(|l| symlink_changed_since(l, start_time));
    target_changed || link_changed
  } else {
    leaf_changed(logical_path)
  }
}

#[cfg(test)]
mod tests {
  use rspack_paths::InternedPath;

  use super::*;

  #[tokio::test]
  async fn test_scan() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let path_manager = PathManager::default();

    let files = (
      vec![current_dir.join("___test_file.txt").into()].into_iter(),
      vec![].into_iter(),
    );

    let dirs = (
      vec![current_dir.join("___test_dir/a/b/c").into()].into_iter(),
      vec![].into_iter(),
    );

    let missing = (
      vec![current_dir.join("___missing_file.txt").into()].into_iter(),
      vec![].into_iter(),
    );
    path_manager.update(files, dirs, missing).await.unwrap();

    let (tx, mut _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut scanner = Scanner::new(tx, Arc::new(path_manager));

    let collector = tokio::spawn(async move {
      let mut collected_events = Vec::new();
      while let Some(event) = _rx.recv().await {
        collected_events.push(event);
      }
      collected_events
    });

    scanner.scan(SystemTime::now());
    // Simulate scanner dropping to trigger the end of the channel
    scanner.close();

    let collected_events = collector.await.unwrap();
    println!("Collected events: {collected_events:?}");
    assert_eq!(collected_events.len(), 2);

    assert!(collected_events.contains(&vec![FsEvent {
      path: InternedPath::from(current_dir.join("___test_file.txt")),
      kind: FsEventKind::Remove
    }]));
    assert!(collected_events.contains(&vec![FsEvent {
      path: InternedPath::from(current_dir.join("___test_dir/a/b/c")),
      kind: FsEventKind::Remove,
    }]));
  }

  /// Park a file's mtime in the past so a scan-time stat sees it as unchanged
  /// regardless of the process-global `FS_ACCURACY`.
  fn set_mtime_in_past(path: impl AsRef<std::path::Path>, ago: std::time::Duration) {
    let file = std::fs::File::options()
      .write(true)
      .open(path)
      .expect("open for set_modified");
    file
      .set_modified(SystemTime::now() - ago)
      .expect("set_modified");
  }

  /// The scan reports a registered file changed at or after `start_time` from a
  /// fresh disk stat, and leaves an unchanged (old-mtime) file alone.
  #[tokio::test]
  async fn scan_reports_file_changed_since_start_time() {
    use std::{collections::HashSet, time::Duration};

    let dir = tempfile::tempdir().expect("create temp dir");
    let changed = InternedPath::from(dir.path().join("changed.js").as_path());
    let unchanged = InternedPath::from(dir.path().join("unchanged.js").as_path());
    std::fs::write(changed.as_ref(), b"a").expect("write changed");
    std::fs::write(unchanged.as_ref(), b"b").expect("write unchanged");
    // `unchanged` is parked well before start_time; `changed` keeps its ~now mtime.
    set_mtime_in_past(unchanged.as_ref(), Duration::from_secs(3600));

    let path_manager = Arc::new(PathManager::default());
    path_manager
      .update(
        (
          vec![changed.clone(), unchanged.clone()].into_iter(),
          std::iter::empty(),
        ),
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
      )
      .await
      .expect("register files");

    // start_time sits before `changed`'s mtime but after `unchanged`'s.
    let start_time = SystemTime::now() - Duration::from_secs(5);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut scanner = Scanner::new(tx, path_manager.clone());
    scanner.scan(start_time);
    scanner.close();

    let mut changed_paths = HashSet::new();
    while let Some(batch) = rx.recv().await {
      for ev in batch {
        if ev.kind == FsEventKind::Change {
          changed_paths.insert(ev.path);
        }
      }
    }

    assert!(
      changed_paths.contains(&changed),
      "a file changed at/after start_time must be reported",
    );
    assert!(
      !changed_paths.contains(&unchanged),
      "a file unchanged before start_time must not be reported",
    );
  }

  /// A registered-missing dependency created after `start_time` must be
  /// backfilled as a `Create`; one that never appears must not be reported.
  #[tokio::test]
  async fn scan_backfills_missing_path_created_after_start() {
    use std::{collections::HashSet, time::Duration};

    let dir = tempfile::tempdir().expect("create temp dir");
    let created = InternedPath::from(dir.path().join("created.js").as_path());
    let still_missing = InternedPath::from(dir.path().join("still_missing.js").as_path());

    let path_manager = Arc::new(PathManager::default());
    path_manager
      .update(
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        (
          vec![created.clone(), still_missing.clone()].into_iter(),
          std::iter::empty(),
        ),
      )
      .await
      .expect("register missing deps");

    // start_time is in the past; the missing dep is created "now", after it.
    let start_time = SystemTime::now() - Duration::from_secs(5);
    std::fs::write(created.as_ref(), b"new").expect("create file");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut scanner = Scanner::new(tx, path_manager.clone());
    scanner.scan(start_time);
    scanner.close();

    let mut event_paths = HashSet::new();
    while let Some(batch) = rx.recv().await {
      for ev in batch {
        event_paths.insert(ev.path);
      }
    }

    assert!(
      event_paths.contains(&created),
      "a missing dependency created after start_time must be backfilled",
    );
    assert!(
      !event_paths.contains(&still_missing),
      "a dependency that never appears must not be reported",
    );
  }
}
