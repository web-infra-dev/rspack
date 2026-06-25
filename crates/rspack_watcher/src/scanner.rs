use std::{ops::Deref, sync::Arc, time::SystemTime};

use rspack_paths::{ArcPath, ArcPathDashSet};
use tokio::sync::mpsc::UnboundedSender;

use super::{FsEvent, FsEventKind, PathManager};
use crate::{EventBatch, time_info};

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
      tokio::spawn(async move {
        _ = scan_path_missing(&files, &missing, &files_tx);
        _ = scan_path_events(
          &files,
          |p| changed_since(p, start_time),
          FsEventKind::Change,
          &files_tx,
        );
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
        _ = scan_path_missing(&directories, &missing, &dirs_tx);
        _ = scan_path_events(
          &directories,
          |p| changed_since(p, start_time),
          FsEventKind::Change,
          &dirs_tx,
        );
      });

      // Backfill registered-missing dependencies created in the gap before this
      // `watch()` registration: a `Create` once the file appears on disk.
      let missing_added = accessor
        .missing()
        .1
        .iter()
        .map(|p| p.deref().clone())
        .collect::<Vec<_>>();
      tokio::spawn(async move {
        _ = scan_path_events(
          &missing_added,
          |p| changed_since(p, start_time),
          FsEventKind::Create,
          &tx,
        );
      });
    }
  }

  pub fn close(&mut self) {
    // Close the scanner by dropping the sender
    self.tx.take();
  }
}

fn scan_path_missing(
  paths: &[ArcPath],
  missing: &ArcPathDashSet,
  tx: &UnboundedSender<EventBatch>,
) -> bool {
  let remove_event = paths
    .iter()
    .filter(|path| !path.exists() && !missing.contains(*path))
    .cloned()
    .map(|path| FsEvent {
      path,
      kind: FsEventKind::Remove,
    })
    .collect::<Vec<_>>();
  if remove_event.is_empty() {
    return true;
  }
  tx.send(remove_event).is_ok()
}

fn scan_path_events(
  paths: &[ArcPath],
  selected: impl Fn(&ArcPath) -> bool,
  kind: FsEventKind,
  tx: &UnboundedSender<EventBatch>,
) -> bool {
  let events = paths
    .iter()
    .filter(|path| selected(path))
    .cloned()
    .map(|path| FsEvent { path, kind })
    .collect::<Vec<_>>();

  if events.is_empty() {
    return true;
  }
  tx.send(events).is_ok()
}

/// Whether `path`'s current on-disk mtime is at or after `start_time`, using
/// watchpack's accuracy padding ([`time_info::safe_time`]) so a change hidden by
/// coarse mtime granularity is still caught. A failed stat (missing/unreadable)
/// counts as unchanged.
fn changed_since(path: &ArcPath, start_time: SystemTime) -> bool {
  let Ok(mtime) = path
    .metadata()
    .and_then(|m| m.modified().or_else(|_| m.created()))
  else {
    return false;
  };
  time_info::safe_time(time_info::system_time_to_millis(mtime))
    >= time_info::system_time_to_millis(start_time)
}

#[cfg(test)]
mod tests {
  use rspack_paths::ArcPath;

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
    path_manager.update(files, dirs, missing).unwrap();

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
      path: ArcPath::from(current_dir.join("___test_file.txt")),
      kind: FsEventKind::Remove
    }]));
    assert!(collected_events.contains(&vec![FsEvent {
      path: ArcPath::from(current_dir.join("___test_dir/a/b/c")),
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
    let changed = ArcPath::from(dir.path().join("changed.js").as_path());
    let unchanged = ArcPath::from(dir.path().join("unchanged.js").as_path());
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
    let created = ArcPath::from(dir.path().join("created.js").as_path());
    let still_missing = ArcPath::from(dir.path().join("still_missing.js").as_path());

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
