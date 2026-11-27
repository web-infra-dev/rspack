use std::{sync::Arc, time::SystemTime};

use rspack_paths::InternedPathMap;

use super::{FsEvent, PathManager};
use crate::{
  directory_watcher::DirectoryWatcher,
  paths::{Backfill, Registration},
};

pub(crate) type BackfillSink = Arc<dyn Fn(FsEvent, Backfill) + Send + Sync>;

pub struct Scanner {
  path_manager: Arc<PathManager>,
  sink: Option<BackfillSink>,
}

impl Scanner {
  pub(crate) fn with_sink(sink: BackfillSink, path_manager: Arc<PathManager>) -> Self {
    Self {
      path_manager,
      sink: Some(sink),
    }
  }

  #[cfg(test)]
  fn new(
    tx: tokio::sync::mpsc::UnboundedSender<crate::EventBatch>,
    path_manager: Arc<PathManager>,
  ) -> Self {
    Self::with_sink(
      Arc::new(move |event, _| {
        let _ = tx.send(vec![event]);
      }),
      path_manager,
    )
  }

  pub(crate) fn prepare(&self, start: SystemTime) -> impl Future<Output = ()> + Send + use<> {
    let mut dirs = InternedPathMap::<Vec<Registration>>::default();
    self.path_manager.added_registrations(|registration| {
      dirs
        .entry(DirectoryWatcher::owner(&registration.path))
        .or_default()
        .push(registration);
    });
    let mut passes = Vec::with_capacity(dirs.len());
    if let Some(sink) = &self.sink
      && !dirs.is_empty()
    {
      let batch = crate::observation::ActiveWalk::start(&self.path_manager.watchers.observations);
      for (dir, entries) in dirs {
        let Some(queue) = self.path_manager.watchers.directory_scan(&dir) else {
          continue;
        };
        let paths = Arc::clone(&self.path_manager);
        let sink = Arc::clone(sink);
        let batch = Arc::clone(&batch);
        passes.push(queue.targeted(move || {
          for entry in entries {
            if let Some((event, backfill)) = paths.scan_registered(entry, start, &batch) {
              sink(event, backfill);
            }
          }
        }));
      }
    }
    async move {
      futures::future::join_all(passes).await;
    }
  }

  #[cfg(any(test, feature = "testing"))]
  pub(crate) fn scan(&self, start: SystemTime) {
    tokio::spawn(self.prepare(start));
  }

  pub fn close(&mut self) {
    self.sink.take();
  }
}

#[cfg(test)]
use crate::FsEventKind;

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
