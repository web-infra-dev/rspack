use std::{sync::Arc, time::SystemTime};

use rspack_paths::InternedPath;
use rspack_util::time::system_time_to_millis;

use super::{FsEvent, FsEventKind, PathManager};
use crate::{
  observation::{Mutation, RegisteredChange, Registration, Scan, ScanEpoch},
  time::{mtime_accuracy, mtime_safe_time},
  trigger::Trigger,
};

pub(super) trait ScanSink: Send + Sync {
  fn send(&self, event: FsEvent, change: RegisteredChange);
}

impl ScanSink for Arc<Trigger> {
  fn send(&self, event: FsEvent, change: RegisteredChange) {
    self.on_scan_event(event.path, event.kind, change);
  }
}

#[cfg(any(test, feature = "testing"))]
impl ScanSink for tokio::sync::mpsc::UnboundedSender<super::EventBatch> {
  fn send(&self, event: FsEvent, _: RegisteredChange) {
    let _ = self.send(vec![event]);
  }
}

pub struct Scanner<S = Arc<Trigger>> {
  path_manager: Arc<PathManager>,
  sink: Option<S>,
}

impl<S: ScanSink + Clone + 'static> Scanner<S> {
  pub fn new(sink: S, path_manager: Arc<PathManager>) -> Self {
    Self {
      path_manager,
      sink: Some(sink),
    }
  }

  /// Backfill after the OS watch has been installed. A later watch cycle may
  /// have replaced the subscription by the time a spawned job finishes.
  pub fn scan(&self, start_time: SystemTime) {
    for job in self.prepare(start_time) {
      tokio::spawn(async move {
        job.read()();
      });
    }
  }

  pub(super) fn prepare(&self, start_time: SystemTime) -> Vec<ScanJob<S>> {
    let Some(sink) = &self.sink else {
      return Vec::new();
    };
    let accessor = self.path_manager.access();
    if accessor.files().1.is_empty()
      && accessor.directories().1.is_empty()
      && accessor.missing().1.is_empty()
    {
      return Vec::new();
    }
    let epoch = self.path_manager.start_scan();
    [
      (Registration::File, accessor.files().1),
      (Registration::Context, accessor.directories().1),
      (Registration::Missing, accessor.missing().1),
    ]
    .into_iter()
    .filter(|(_, added)| !added.is_empty())
    .map(|(kind, added)| ScanJob {
      kind,
      start_time: system_time_to_millis(start_time),
      paths: added.iter().map(|p| p.clone()).collect(),
      manager: Arc::clone(&self.path_manager),
      sink: sink.clone(),
      epoch: Arc::clone(&epoch),
    })
    .collect()
  }

  pub fn close(&mut self) {
    self.sink.take();
  }
}

pub(super) struct ScanJob<S> {
  kind: Registration,
  start_time: u64,
  paths: Vec<InternedPath>,
  manager: Arc<PathManager>,
  sink: S,
  epoch: Arc<ScanEpoch>,
}

struct PathStatus {
  present: bool,
  exists: bool,
  symlink: bool,
  mtime: Option<SystemTime>,
}

impl PathStatus {
  fn read(path: &InternedPath, kind: Registration) -> Self {
    // A dangling link exists as an explicit file. Other dependencies and the mtime check follow it.
    let own = std::fs::symlink_metadata(path).ok();
    let exists = own.is_some();
    let symlink = own.as_ref().is_some_and(|m| m.is_symlink());
    let target = if symlink { path.metadata().ok() } else { own };
    let present = matches!(kind, Registration::File) && exists || target.is_some();
    let mtime = target.and_then(|m| {
      let modified = m.modified();
      if let Ok(time) = modified {
        mtime_accuracy(time);
      }
      modified.or_else(|_| m.created()).ok()
    });
    Self {
      present,
      exists,
      symlink,
      mtime,
    }
  }
}

impl<S: ScanSink> ScanJob<S> {
  pub(super) fn read(mut self) -> impl FnOnce() {
    let mut absent = Vec::new();
    let mut present = Vec::new();
    for path in std::mem::take(&mut self.paths) {
      let status = PathStatus::read(&path, self.kind);
      let entries = if status.present {
        &mut present
      } else {
        &mut absent
      };
      entries.push((path, status));
    }
    move || {
      // Report missing paths before changes, as in the original backfill.
      for (path, status) in absent.into_iter().chain(present) {
        self.commit(path, status);
      }
    }
  }

  fn commit(&self, path: InternedPath, mut status: PathStatus) {
    let scan = Scan::new(&self.epoch, self.kind, &path);
    let mut rechecked = None;
    loop {
      let retry = self
        .manager
        .observe(&path, Some(&scan), |state| {
          let newer = match state.newer_scan(&path, &scan) {
            // The directory walk saw a symlink and recorded its own time. An
            // explicit dependency needs a fresh stat of the target.
            Some((version, None)) if rechecked != Some(version) => {
              return (Some(version), Mutation::Unchanged);
            }
            Some((_, time)) => time,
            None => None,
          };
          let mtime = newer.or(status.mtime);
          let changed = mtime.is_some_and(|time| mtime_safe_time(time) >= self.start_time);
          let accessor = self.manager.access();
          let event = match (self.kind, mtime) {
            (Registration::Missing | Registration::File, Some(mtime)) if changed => {
              self.manager.set_reported_file_time(&path, mtime);
              Some(if matches!(self.kind, Registration::File) {
                FsEventKind::Change
              } else {
                FsEventKind::Create
              })
            }
            (Registration::Missing, Some(mtime)) => {
              self.manager.set_file_time(&path, mtime, true, true);
              None
            }
            (Registration::Missing, _) => None,
            (Registration::Context, _) if changed => Some(FsEventKind::Change),
            _ if !status.present
              && newer.is_none()
              && !accessor.missing().0.contains(&path)
              && !(matches!(self.kind, Registration::File)
                && self.manager.is_known_absent(&path)) =>
            {
              Some(FsEventKind::Remove)
            }
            _ => None,
          };
          if let Some(kind) = event {
            self.sink.send(
              FsEvent {
                path: path.clone(),
                kind,
              },
              state.queue_notification(&scan),
            );
          }
          let mutation = if status.exists || newer.is_some() {
            Mutation::Scan(newer.or(status.mtime.filter(|_| !status.symlink)))
          } else {
            Mutation::Subtree
          };
          (None, mutation)
        })
        .flatten();
      let Some(version) = retry else {
        return;
      };
      status = PathStatus::read(&path, self.kind);
      rechecked = Some(version);
    }
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
