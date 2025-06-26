use std::{ops::Deref, sync::Arc};

use tokio::sync::mpsc::UnboundedSender;

use super::{FsEvent, FsEventKind, PathManager};

// Scaner will scann the path whether it is exist or not in disk on initialization
pub struct Scaner {
  path_manager: Arc<PathManager>,
  tx: Option<UnboundedSender<FsEvent>>,
}

impl Scaner {
  /// Creates a new `Scanner` that will send events to the provided sender when paths are scanned.
  pub fn new(tx: UnboundedSender<FsEvent>, path_manager: Arc<PathManager>) -> Self {
    Self {
      path_manager,
      tx: Some(tx),
    }
  }

  /// Scans the registered paths and sends delete events for any files or directories that no longer exist.
  /// align watchpack action: https://github.com/webpack/watchpack/blob/v2.4.4/lib/DirectoryWatcher.js#L565-L568
  pub fn scan(&self) {
    let accessor = self.path_manager.access();

    for file in accessor.files().iter() {
      let filepath = file.deref();
      if !filepath.exists() {
        if let Some(tx) = &self.tx {
          // If the file does not exist, send a delete event
          let _ = tx.send(FsEvent {
            path: filepath.clone(),
            kind: FsEventKind::Remove,
          });
        }
      }
    }

    for dir in accessor.directories().iter() {
      let dirpath = dir.deref();
      if !dirpath.exists() {
        if let Some(tx) = &self.tx {
          // If the directory does not exist, send a delete event
          let _ = tx.send(FsEvent {
            path: dirpath.clone(),
            kind: FsEventKind::Remove,
          });
        }
      }
    }
  }

  pub fn close(&mut self) {
    // Close the scanner by dropping the sender
    self.tx.take();
  }
}

#[cfg(test)]
mod tests {
  use dashmap::DashSet as HashSet;
  use rspack_paths::ArcPath;

  use super::*;

  #[tokio::test]
  async fn test_scan() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let files = HashSet::new();
    files.insert(ArcPath::from(current_dir.join("___test_file.txt")));

    let directories = HashSet::new();
    directories.insert(ArcPath::from(current_dir.join("___test_dir/a/b/c")));

    let missing = HashSet::new();
    missing.insert(ArcPath::from(current_dir.join("___missing_file.txt")));

    let mut path_manager = PathManager::new(None);
    path_manager.files.extend(files);
    path_manager.directories.extend(directories);
    path_manager.missing.extend(missing);

    let (tx, mut _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut scanner = Scaner::new(tx, Arc::new(path_manager));

    let collector = tokio::spawn(async move {
      let mut collected_events = Vec::new();
      while let Some(event) = _rx.recv().await {
        collected_events.push(event);
      }
      collected_events
    });

    scanner.scan();
    // Simulate scanner dropping to trigger the end of the channel
    scanner.close();

    let collected_events = collector.await.unwrap();
    assert_eq!(collected_events.len(), 2);

    assert!(collected_events.contains(&FsEvent {
      path: ArcPath::from(current_dir.join("___test_file.txt")),
      kind: FsEventKind::Remove
    }));
    assert!(collected_events.contains(&FsEvent {
      path: ArcPath::from(current_dir.join("___test_dir/a/b/c")),
      kind: FsEventKind::Remove,
    }));
  }
}
