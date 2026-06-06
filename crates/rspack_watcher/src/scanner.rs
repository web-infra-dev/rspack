use std::{ops::Deref, sync::Arc, time::SystemTime};

use rspack_paths::{ArcPath, ArcPathDashSet};
use tokio::sync::mpsc::UnboundedSender;

use super::{FsEvent, FsEventKind, PathManager};
use crate::EventBatch;

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

  /// Scans the registered paths and sends delete events for any files or directories that no longer exist.
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
      let _tx = tx.clone();
      let pm = self.path_manager.clone();
      tokio::spawn(async move {
        _ = scan_path_missing(&files, &missing, &_tx);
        _ = scan_path_events(
          &files,
          |p| pm.file_changed_since(p, start_time),
          FsEventKind::Change,
          &_tx,
        );
      });

      let directories = accessor
        .directories()
        .1
        .iter()
        .map(|file| file.deref().clone())
        .collect::<Vec<_>>();
      let missing = accessor.missing().0.clone();
      let _tx = tx.clone();
      let pm = self.path_manager.clone();
      tokio::spawn(async move {
        _ = scan_path_missing(&directories, &missing, &_tx);
        _ = scan_path_events(
          &directories,
          |p| pm.dir_changed_since(p, start_time),
          FsEventKind::Change,
          &_tx,
        );
      });

      // Backfill registered-missing dependencies that were created in the gap
      // before this `watch()` registration — a `Create` whose disk mtime
      // (padded by `accuracy`) lands at or after `start_time`. Absorbs
      // for_rstest B4 over `missing.added`.
      let missing_added = accessor
        .missing()
        .1
        .iter()
        .map(|p| p.deref().clone())
        .collect::<Vec<_>>();
      let pm = self.path_manager.clone();
      tokio::spawn(async move {
        _ = scan_path_events(
          &missing_added,
          |p| pm.missing_created_since(p, start_time),
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

  /// The scan must report a file as changed from its watchpack-style
  /// `safe_time`, not from a fresh disk-mtime read. Both files have a disk
  /// mtime *older* than `start_time` (so a raw-mtime scan reports neither), yet
  /// only `changed` recorded a live event after `start_time` — its `safe_time`
  /// crosses `start_time` and it alone must be reported. This is the
  /// granularity-safe behavior raw mtime misses.
  #[tokio::test]
  async fn scan_reports_change_from_safe_time_not_disk_mtime() {
    use std::{collections::HashSet, thread::sleep, time::Duration};

    let dir = tempfile::tempdir().expect("create temp dir");
    let changed = ArcPath::from(dir.path().join("changed.js").as_path());
    let unchanged = ArcPath::from(dir.path().join("unchanged.js").as_path());
    std::fs::write(changed.as_ref(), b"a").expect("write changed");
    std::fs::write(unchanged.as_ref(), b"b").expect("write unchanged");

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

    // start_time sits *after* both files were created on disk, so their disk
    // mtimes are already < start_time.
    sleep(Duration::from_millis(20));
    let start_time = SystemTime::now();
    sleep(Duration::from_millis(20));

    // Only `changed` records a live event after start_time.
    path_manager.set_event_file_time(&changed, SystemTime::now());

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
      "file whose safe_time >= start_time must be reported changed",
    );
    assert!(
      !changed_paths.contains(&unchanged),
      "file with no post-start activity must not be reported changed",
    );
  }

  /// A missing dependency created in the gap after `start_time` must be
  /// backfilled (absorbs for_rstest B4 over `missing.added`); one that never
  /// appears must not be reported.
  #[tokio::test]
  async fn scan_backfills_missing_path_created_after_start() {
    use std::{collections::HashSet, thread::sleep, time::Duration};

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

    let start_time = SystemTime::now();
    sleep(Duration::from_millis(20));
    // The missing dependency is created after start_time.
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
