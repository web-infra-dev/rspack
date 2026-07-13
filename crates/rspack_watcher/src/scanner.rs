use std::{ops::Deref, sync::Arc, time::SystemTime};

use rspack_paths::ArcPath;
use tokio::sync::mpsc::UnboundedSender;

use super::{FsEvent, FsEventKind, PathManager};
use crate::{EventBatch, time_info};

// Scanner inspects registered paths at watch startup.
//
// Two responsibilities:
// 1. `reclassify_missing` — for file/directory deps that are not present on
//    disk, reclassify them into the `missing` tracker. Runs synchronously
//    BEFORE the analyzer so it sees the updated tracker and watches those paths
//    for future creation — the same way watchpack handles absent
//    `fileDependencies`. No Remove event is emitted here; changes for paths
//    served outside the real fs (e.g. virtual modules) are delivered through
//    `FsWatcher::trigger_event` by the owning plugin.
// 2. `scan` — synthesize the events the live watch could not deliver yet: a
//    `Change` for a registered path modified since `start_time`, and a `Create`
//    for a registered-missing dependency that has appeared. Dispatched to the
//    tokio runtime and runs AFTER the OS watch is active (#14210), so a change
//    landing before the watch is on disk and caught here.
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

  /// Reclassify registered file/directory deps absent from disk into the
  /// `missing` tracker. Synchronous so the analyzer, which runs immediately
  /// after, sees the updated tracker and watches those paths for creation
  /// instead of treating them as removed.
  pub fn reclassify_missing(&self) {
    let accessor = self.path_manager.access();
    let files = accessor
      .files()
      .1
      .iter()
      .map(|file| file.deref().clone())
      .collect::<Vec<_>>();
    let directories = accessor
      .directories()
      .1
      .iter()
      .map(|dir| dir.deref().clone())
      .collect::<Vec<_>>();

    scan_path_missing(&files, &self.path_manager);
    scan_path_missing(&directories, &self.path_manager);
  }

  /// Synthesizes the events the live watch could not deliver yet: a `Change` for
  /// a file/directory changed since `start_time`, and a `Create` for a
  /// registered-missing dependency that has appeared. Change is judged from a
  /// fresh, accuracy-padded mtime read ([`changed_since`]) — the scan runs after
  /// the OS watch is active (#14210), so a change landing before the watch is on
  /// disk and caught here. Absent registered paths are not reported as `Remove`;
  /// they are reclassified into `missing` by [`Scanner::reclassify_missing`].
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
      let files_tx = tx.clone();
      tokio::spawn(async move {
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
      let dirs_tx = tx.clone();
      tokio::spawn(async move {
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

/// Reclassify paths that are absent from the real filesystem as missing deps.
/// Paths already tracked as missing are skipped, so `missing.added` keeps
/// meaning "newly missing" for the analyzer and for the `Create` backfill in
/// [`Scanner::scan`].
/// No events are emitted: the watcher waits for them to appear (either via an
/// OS event on the watched parent directory or an explicit `trigger_event`
/// from the owning plugin).
fn scan_path_missing(paths: &[ArcPath], path_manager: &PathManager) {
  let accessor = path_manager.access();
  let missing = accessor.missing().0;
  for path in paths {
    if !path.exists() && !missing.contains(path) {
      path_manager.promote_to_missing(path.clone());
    }
  }
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
  async fn test_scan_missing_paths_are_promoted_to_missing() {
    // Paths absent from disk should be reclassified into the `missing`
    // tracker, not reported as Remove events.
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let path_manager = PathManager::default();

    let ghost_file: ArcPath = current_dir.join("___ghost_file.txt").into();
    let ghost_dir: ArcPath = current_dir.join("___ghost_dir/a/b/c").into();

    let files = (vec![ghost_file.clone()].into_iter(), vec![].into_iter());
    let dirs = (vec![ghost_dir.clone()].into_iter(), vec![].into_iter());
    let missing = (vec![].into_iter(), vec![].into_iter());
    path_manager.update(files, dirs, missing).unwrap();

    let path_manager = Arc::new(path_manager);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut scanner = Scanner::new(tx, Arc::clone(&path_manager));

    let collector = tokio::spawn(async move {
      let mut collected = Vec::new();
      while let Some(event) = rx.recv().await {
        collected.push(event);
      }
      collected
    });

    scanner.reclassify_missing();
    scanner.scan(SystemTime::now());
    scanner.close();

    let collected = collector.await.unwrap();
    assert!(
      collected
        .iter()
        .flatten()
        .all(|event| event.kind != FsEventKind::Remove),
      "scan should not emit Remove for missing paths, got: {collected:?}"
    );

    let accessor = path_manager.access();
    let missing_all = accessor.missing().0;
    assert!(
      missing_all.contains(&ghost_file),
      "ghost file should be promoted to missing tracker"
    );
    assert!(
      missing_all.contains(&ghost_dir),
      "ghost directory should be promoted to missing tracker"
    );
  }

  #[tokio::test]
  async fn test_scan_change_emits_for_fresh_file() {
    // A real file whose mtime is after start_time should emit Change.
    let tmp = tempfile::TempDir::new().unwrap();
    let file_path = tmp.path().join("fresh.txt");
    std::fs::write(&file_path, "hello").unwrap();

    let start_time = SystemTime::now() - std::time::Duration::from_secs(10);

    let path_manager = PathManager::default();
    let files = (
      vec![ArcPath::from(file_path.clone())].into_iter(),
      vec![].into_iter(),
    );
    let dirs = (vec![].into_iter(), vec![].into_iter());
    let missing = (vec![].into_iter(), vec![].into_iter());
    path_manager.update(files, dirs, missing).unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut scanner = Scanner::new(tx, Arc::new(path_manager));

    let collector = tokio::spawn(async move {
      let mut collected = Vec::new();
      while let Some(event) = rx.recv().await {
        collected.push(event);
      }
      collected
    });

    scanner.scan(start_time);
    scanner.close();

    let collected = collector.await.unwrap();
    assert!(
      collected
        .iter()
        .flatten()
        .any(|event| event.kind == FsEventKind::Change
          && event.path == ArcPath::from(file_path.clone())),
      "scan should emit Change for file with mtime after start_time, got: {collected:?}"
    );
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
