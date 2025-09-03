use std::{ops::Deref, sync::Arc, time::SystemTime};

use rspack_paths::ArcPath;
use tokio::sync::mpsc::UnboundedSender;

use super::{FsEvent, FsEventKind, PathManager};
use crate::watcher::EventBatch;

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
    let accessor = self.path_manager.access();

    // only apply for added files
    let files = accessor.files().1.clone();
    let missing = accessor.missing().0.clone();
    let start_time = Arc::new(start_time);
    let start_time_clone = Arc::clone(&start_time);
    let _tx = self.tx.clone();
    tokio::spawn(async move {
      for file in files.iter() {
        let filepath = file.deref();
        if let Some(tx) = &_tx {
          if !filepath.exists() && !missing.contains(filepath) {
            // If the file does not exist, send a delete event
            let _ = tx.send(vec![FsEvent {
              path: filepath.clone(),
              kind: FsEventKind::Remove,
            }]);
          }

          if check_path_metadata(filepath, &start_time_clone) {
            // If the file has been modified since the start time, send a change event
            let _ = tx.send(vec![FsEvent {
              path: filepath.clone(),
              kind: FsEventKind::Change,
            }]);
          }
        }
      }
    });

    let directories = accessor.directories().1.clone();
    let missing = accessor.missing().0.clone();
    let _tx = self.tx.clone();
    tokio::spawn(async move {
      for dir in directories.iter() {
        let dirpath = dir.deref();
        if let Some(tx) = &_tx {
          if !dirpath.exists() && !missing.contains(dirpath) {
            // If the directory does not exist, send a delete event
            let _ = tx.send(vec![FsEvent {
              path: dirpath.clone(),
              kind: FsEventKind::Remove,
            }]);
          }

          if check_path_metadata(dirpath, &start_time) {
            // If the directory has been modified since the start time, send a change event
            let _ = tx.send(vec![FsEvent {
              path: dirpath.clone(),
              kind: FsEventKind::Change,
            }]);
          }
        }
      }
    });
  }

  pub fn close(&mut self) {
    // Close the scanner by dropping the sender
    self.tx.take();
  }
}

fn check_path_metadata(filepath: &ArcPath, start_time: &SystemTime) -> bool {
  if let Ok(m_time) = filepath
    .metadata()
    .and_then(|metadata| metadata.modified().or(metadata.created()))
  {
    *start_time < m_time
  } else {
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
