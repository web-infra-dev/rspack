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

      eprintln!(
        "[WATCHER_DEBUG] Scanner::scan() - starting file scan for {} files, start_time: {:?}",
        files.len(),
        start_time
      );

      tokio::spawn(async move {
        eprintln!("[WATCHER_DEBUG] Scanner - file scan task started");
        if !scan_path_missing(&files, &missing, &_tx) {
          eprintln!(
            "[WATCHER_DEBUG] Scanner - WARNING: scan_path_missing for files failed to send events"
          );
        }
        if !scan_path_changed(&files, &start_time, &_tx) {
          eprintln!(
            "[WATCHER_DEBUG] Scanner - WARNING: scan_path_changed for files failed to send events"
          );
        }
        eprintln!("[WATCHER_DEBUG] Scanner - file scan task completed");
      });

      let directories = accessor
        .directories()
        .1
        .iter()
        .map(|file| file.deref().clone())
        .collect::<Vec<_>>();
      let missing = accessor.missing().0.clone();
      let _tx = self.tx.clone();

      eprintln!(
        "[WATCHER_DEBUG] Scanner::scan() - starting directory scan for {} directories",
        directories.len()
      );

      tokio::spawn(async move {
        eprintln!("[WATCHER_DEBUG] Scanner - directory scan task started");
        if !scan_path_missing(&directories, &missing, &tx) {
          eprintln!(
            "[WATCHER_DEBUG] Scanner - WARNING: scan_path_missing for directories failed to send events"
          );
        }
        if !scan_path_changed(&directories, &start_time, &tx) {
          eprintln!(
            "[WATCHER_DEBUG] Scanner - WARNING: scan_path_changed for directories failed to send events"
          );
        }
        eprintln!("[WATCHER_DEBUG] Scanner - directory scan task completed");
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
  eprintln!(
    "[WATCHER_DEBUG] scan_path_missing - Found {} missing paths",
    remove_event.len()
  );
  for event in &remove_event {
    eprintln!("[WATCHER_DEBUG]   - Missing: {:?}", event.path);
  }
  tx.send(remove_event).is_ok()
}

fn scan_path_changed(
  paths: &[ArcPath],
  start_time: &SystemTime,
  tx: &UnboundedSender<EventBatch>,
) -> bool {
  eprintln!(
    "[WATCHER_DEBUG] scan_path_changed - Checking {} paths against start_time: {:?}",
    paths.len(),
    start_time
  );
  let changed_event = paths
    .iter()
    .filter(|path| check_path_metadata(path, start_time))
    .cloned()
    .map(|path| FsEvent {
      path,
      kind: FsEventKind::Change,
    })
    .collect::<Vec<_>>();

  if changed_event.is_empty() {
    eprintln!("[WATCHER_DEBUG] scan_path_changed - No changed files detected");
    return true;
  }
  eprintln!(
    "[WATCHER_DEBUG] scan_path_changed - Found {} changed files",
    changed_event.len()
  );
  for event in &changed_event {
    eprintln!("[WATCHER_DEBUG]   - Changed: {:?}", event.path);
  }
  tx.send(changed_event).is_ok()
}

fn check_path_metadata(filepath: &ArcPath, start_time: &SystemTime) -> bool {
  if let Ok(m_time) = filepath
    .metadata()
    .and_then(|metadata| metadata.modified().or(metadata.created()))
  {
    // Fix: Use <= instead of < to catch files modified at exactly start_time
    // This is critical for CI environments where timestamp precision may be low
    let is_changed = *start_time <= m_time;
    if is_changed {
      let delta = m_time
        .duration_since(*start_time)
        .ok()
        .map(|d| d.as_millis())
        .unwrap_or(0);
      eprintln!(
        "[WATCHER_DEBUG] check_path_metadata - File changed: {:?}, start_time: {:?}, m_time: {:?}, delta: {}ms",
        filepath, start_time, m_time, delta
      );
    } else {
      eprintln!(
        "[WATCHER_DEBUG] check_path_metadata - File NOT changed: {:?}, start_time: {:?}, m_time: {:?}",
        filepath, start_time, m_time
      );
    }
    is_changed
  } else {
    eprintln!(
      "[WATCHER_DEBUG] check_path_metadata - Failed to get metadata for: {:?}",
      filepath
    );
    false
  }
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
    println!("Collected events: {:?}", collected_events);
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
}
