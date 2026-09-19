use std::{ops::Deref, sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathDashSet};
use rspack_util::time::{mtime_safe_time, system_time_to_millis};
use tokio::sync::mpsc::UnboundedSender;

use super::{EventBatch, FsEvent, FsEventKind, PathManager};
use crate::paths::disk_mtime;

// Scanner will scann the path whether it is exist or not in disk on initialization
pub struct Scanner {
  path_manager: Arc<PathManager>,
  tx: Option<UnboundedSender<EventBatch>>,
}

impl Scanner {
  /// Creates a new `Scanner` that will send events to the provided sender when paths are scanned.
  pub fn new(tx: UnboundedSender<EventBatch>, path_manager: Arc<PathManager>) -> Self {
    Self {
      path_manager,
      tx: Some(tx),
    }
  }

  /// Synthesizes the events the live watch could not deliver yet: a `Remove` for
  /// a registered path gone from disk, a `Change` for a file/directory changed
  /// since `start_time`, and a `Create` for a registered-missing dependency that
  /// has appeared. Change is judged from a fresh, accuracy-padded mtime read
  /// ([`changed_since`]) — the scan runs after the OS watch is active (#14210),
  /// so a change landing before the watch is on disk and caught here.
  /// align watchpack action: https://github.com/webpack/watchpack/blob/v2.4.4/lib/DirectoryWatcher.js#L565-L568
  pub fn scan(&self, start_time: SystemTime) {
    if let Some(tx) = self.tx.clone() {
      let accessor = self.path_manager.access();
      // only apply for added files
      let files = accessor
        .files()
        .1
        .iter()
        .map(|file| file.deref().clone())
        .collect::<Vec<_>>();
      let missing = accessor.missing().0.clone();
      let files_tx = tx.clone();
      let files_path_manager = Arc::clone(&self.path_manager);
      tokio::spawn(async move {
        let removed = absent_paths(&files, &missing);
        let changed = files
          .iter()
          .filter(|p| changed_since(p, start_time))
          .cloned()
          .collect::<Vec<_>>();
        // These backfills bypass `Trigger`, which is what normally keeps the
        // file-time records in step with delivered events: a `Remove` drops
        // the record, a `Change` re-reads it if the mtime moved on since it
        // was seeded.
        for path in &removed {
          files_path_manager.remove_file_time(path);
        }
        for path in &changed {
          if let Some(mtime) = disk_mtime(path) {
            files_path_manager.set_file_time(path, mtime, true, true);
          }
        }
        _ = send_events(removed, FsEventKind::Remove, &files_tx);
        _ = send_events(changed, FsEventKind::Change, &files_tx);
      });

      let directories = accessor
        .directories()
        .1
        .iter()
        .map(|file| file.deref().clone())
        .collect::<Vec<_>>();
      let missing = accessor.missing().0.clone();
      let dirs_tx = tx.clone();
      tokio::spawn(async move {
        _ = send_events(
          absent_paths(&directories, &missing),
          FsEventKind::Remove,
          &dirs_tx,
        );
        let changed = directories
          .into_iter()
          .filter(|p| changed_since(p, start_time))
          .collect::<Vec<_>>();
        _ = send_events(changed, FsEventKind::Change, &dirs_tx);
      });

      // Backfill registered-missing dependencies created in the gap before this
      // `watch()` registration: a `Create` once the file appears on disk.
      let missing_added = accessor
        .missing()
        .1
        .iter()
        .map(|p| p.deref().clone())
        .collect::<Vec<_>>();
      let path_manager = Arc::clone(&self.path_manager);
      tokio::spawn(async move {
        let created = missing_added
          .into_iter()
          .filter(|p| changed_since(p, start_time))
          .collect::<Vec<_>>();
        // This backfill bypasses `Trigger`, so record the file time here: it
        // is what lets `collect_time_info_entries` report the dependency as
        // present.
        for path in &created {
          if let Some(mtime) = disk_mtime(path) {
            path_manager.set_file_time(path, mtime, true, false);
          }
        }
        _ = send_events(created, FsEventKind::Create, &tx);
      });
    }
  }

  pub fn close(&mut self) {
    // Close the scanner by dropping the sender
    self.tx.take();
  }
}

/// The registered `paths` gone from disk (a registered-missing path is
/// expected to be absent and is not one of them).
fn absent_paths(paths: &[InternedPath], missing: &InternedPathDashSet) -> Vec<InternedPath> {
  paths
    .iter()
    .filter(|path| !path.exists() && !missing.contains(*path))
    .cloned()
    .collect()
}

/// Sends one `kind` event per path as a single batch; nothing to send counts
/// as sent.
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

/// Whether `path`'s current on-disk mtime is at or after `start_time`, using
/// watchpack's accuracy padding ([`mtime_safe_time`]) so a change hidden by
/// coarse mtime granularity is still caught. A failed stat (missing/unreadable)
/// counts as unchanged.
fn changed_since(path: &InternedPath, start_time: SystemTime) -> bool {
  let Ok(mtime) = path
    .metadata()
    .and_then(|m| m.modified().or_else(|_| m.created()))
  else {
    return false;
  };
  mtime_safe_time(system_time_to_millis(mtime)) >= system_time_to_millis(start_time)
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
