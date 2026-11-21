use std::sync::Arc;

use rspack_paths::{InternedPath, InternedPathDashSet};
use smallvec::SmallVec;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ImmediateEvent, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  paths::PathManager,
  subscriptions::{RequestedPaths, Subscribers},
};
/// `DependencyFinder` provides references to sets of files, directories, and missing paths,
/// allowing efficient lookup and dependency resolution for a given path.
///
/// This struct is typically used to determine which registered dependencies (files, directories,
/// or missing paths) are related to a specific filesystem path, such as when handling file system events.
pub struct DependencyFinder<'a> {
  /// Reference to the set of registered file paths.
  pub files: &'a InternedPathDashSet,
  /// Reference to the set of registered directory paths.
  pub directories: &'a InternedPathDashSet,
  /// Reference to the set of registered missing paths (paths that were expected but not found).
  pub missing: &'a InternedPathDashSet,
}

impl<'a> DependencyFinder<'a> {
  fn visit_subscriptions(
    &self,
    path: &InternedPath,
    mut visit: impl FnMut(&InternedPath, bool, bool),
  ) {
    let target = path.metadata().ok().map(|metadata| metadata.file_type());
    let file = target.is_none_or(|kind| kind.is_file());
    let directory = target.is_none_or(|kind| kind.is_dir());
    if (file && self.contains_file(path)) || (directory && self.contains_directory(path)) {
      visit(path, file, directory);
    }
    for parent in path.ancestors().skip(1).map(InternedPath::from) {
      if self.contains_directory(&parent) {
        visit(&parent, false, true);
      }
    }
  }

  #[cfg(test)]
  fn find_associated_event(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
  ) -> Vec<(InternedPath, FsEventKind)> {
    let mut events = Vec::new();
    self.visit_subscriptions(path, |watched, _, _| {
      events.push((
        watched.clone(),
        if watched == path {
          kind
        } else {
          FsEventKind::Change
        },
      ));
    });
    events
  }

  /// Checks if the given path is registered as a file or missing.
  fn contains_file(&self, path: &InternedPath) -> bool {
    self.files.contains(path) || self.missing.contains(path)
  }

  /// Checks if the given path is registered as a directory or missing.
  fn contains_directory(&self, path: &InternedPath) -> bool {
    self.directories.contains(path) || self.missing.contains(path)
  }

  fn contains_path(&self, path: &InternedPath) -> bool {
    self.files.contains(path) || self.directories.contains(path) || self.missing.contains(path)
  }
}

/// Classifies one raw file system event: drops what the `ignored` filter
/// rejects, normalizes the kind against the disk, and resolves which registered
/// dependencies the event should wake up.
///
/// Deliberately NOT run on the thread that reads events from the OS. One event
/// costs several `stat` calls, and a JS-backed `ignored` predicate costs a full
/// Node event loop turn; doing that inline stalls the reader, and on Linux the
/// inotify queue is bounded — a stalled reader means the kernel silently drops
/// events (`IN_Q_OVERFLOW`).
struct EventProcessor {
  /// Shared reference to the path register, which tracks watched files/directories/missing.
  path_manager: Arc<PathManager>,
  /// Sender for communicating file system events to the watcher executor.
  tx: UnboundedSender<ResolvedEvents>,
  analyzer: Arc<RecommendedAnalyzer>,
}

/// `Trigger` is responsible for sending file system events to the event channel
/// when a relevant file or directory change is detected.
///
/// It only buffers: the OS watcher callback hands an event over and returns
/// immediately, and a dedicated thread drains the buffer one event at a time.
pub struct Trigger {
  pending: UnboundedSender<(InternedPath, FsEventKind, Origin)>,
  pub(crate) analyzer: Arc<RecommendedAnalyzer>,
}

/// Where an event comes from.
#[derive(Clone, Copy)]
enum Origin {
  /// The OS watcher; `appeared` when it created or renamed the path into place.
  Disk { appeared: bool },
  /// An explicit event supplied through `FsWatcher::trigger_event`.
  Injected,
  /// A scan, for what changed while the watch was not active yet: a file a
  /// context's initial scan found changed since the start time, or a
  /// registered path the scanner backfills. Already judged a change, it is
  /// never dropped as a duplicate or as attribute-only.
  Scan,
}

impl Trigger {
  /// Create a new `Trigger` with the given path register and event sender.
  pub fn new(path_manager: Arc<PathManager>, tx: UnboundedSender<ResolvedEvents>) -> Self {
    let (pending, mut incoming) = unbounded_channel::<(InternedPath, FsEventKind, Origin)>();
    let analyzer = Arc::default();
    let processor = EventProcessor {
      path_manager,
      tx,
      analyzer: Arc::clone(&analyzer),
    };

    // One event at a time, in arrival order: the JS `ignored` predicate is
    // awaited, not blocked on, and the next event is not started until the
    // current one has been classified.
    //
    // Ends when the last `Trigger` is dropped and the channel closes.
    std::thread::Builder::new()
      .name("rspack-fs-event-filter".to_string())
      .spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
          .enable_all()
          .build()
          .expect("create watcher event filter runtime");

        runtime.block_on(async move {
          while let Some((path, kind, origin)) = incoming.recv().await {
            processor.process(&path, kind, origin).await;
          }
        });
      })
      .expect("spawn watcher event filter thread");

    Self { pending, analyzer }
  }

  /// Queues an explicit event from `FsWatcher::trigger_event` without blocking.
  pub fn on_event(&self, path: &InternedPath, kind: FsEventKind) {
    self.analyzer.invalidate_path(path);
    let _ = self.pending.send((path.clone(), kind, Origin::Injected));
  }

  /// Preserves whether an OS event created or renamed a path into place,
  /// without treating ordinary directory changes as appearances.
  pub(crate) fn on_disk_event(&self, path: &InternedPath, kind: FsEventKind, appeared: bool) {
    let _ = self
      .pending
      .send((path.clone(), kind, Origin::Disk { appeared }));
  }

  /// A context's initial scan found `path` changed since the start time, while
  /// the watch was not active yet: like watchpack's initial scan, it is
  /// reported as found, never taken for an attribute-only event.
  pub(crate) fn on_scanned_change(&self, path: &InternedPath) {
    let _ = self
      .pending
      .send((path.clone(), FsEventKind::Change, Origin::Scan));
  }

  /// A sender for the scanner's backfilled events. They queue with the live
  /// ones, so whichever is processed last for a path ends as its state on
  /// disk: a removal backfilled for a file back on disk by then is a change.
  pub(crate) fn backfill_sender(&self) -> UnboundedSender<EventBatch> {
    let (tx, mut batches) = unbounded_channel::<EventBatch>();
    let pending = self.pending.clone();
    std::thread::Builder::new()
      .name("rspack-fs-scan-backfill".to_string())
      .spawn(move || {
        while let Some(batch) = batches.blocking_recv() {
          for event in batch {
            let _ = pending.send((event.path, event.kind, Origin::Scan));
          }
        }
      })
      .expect("spawn watcher scan backfill thread");
    tx
  }
}

impl EventProcessor {
  /// Called when a file system event occurs.
  /// Finds all dependencies related to the given path and triggers events for each.
  /// # Example
  /// Consider the following path register:
  ///
  /// ```bash
  /// - /path
  /// - /path/to
  /// - /path/to/file.js
  /// ```
  /// If the file `/path/to/file.js` is changed, the trigger will send an event for the following paths:
  /// - `/path`
  /// - `/path/to`
  async fn process(&self, path: &InternedPath, kind: FsEventKind, origin: Origin) {
    let appeared = matches!(origin, Origin::Disk { appeared: true })
      || (matches!(origin, Origin::Injected) && kind == FsEventKind::Create);
    // Drop events inside ignored subtrees. The recursive-root watch delivers
    // events for unregistered paths (e.g. the build-output dir); left through,
    // `find_associated_event` bubbles them to a registered parent and triggers
    // a spurious rebuild.
    // A scan's event was filtered when the scan found the path.
    if !matches!(origin, Origin::Scan) && self.path_manager.is_ignored_path(path.as_ref()).await {
      return;
    }

    // A watched path that no longer exists on disk is a removal, regardless of
    // how the OS reported the event. macOS FSEvents reports an unlink as a
    // rename (`ModifyKind::Name` → `Change`), so normalize a `Change` whose
    // path is gone into a `Remove`, keeping the event kind consistent with
    // inotify (which already reports `Remove`). A delayed `Create` whose
    // path has already disappeared is also a removal. Done before the stale-event
    // filter below, which only applies to `Change`/`Create`.
    // The reverse holds too: a removal of a path back on disk (unlinked and
    // re-created) ends as the path's final state, like watchpack's lstat on
    // each event. Keep its records unless the absence was observed, so an
    // unchanged aged mtime is suppressed just as it is by watchpack. Explicit and
    // scanned removals already declare the old record invalid.
    let removed = kind == FsEventKind::Remove && !matches!(origin, Origin::Disk { .. });
    let scan_hint = matches!(origin, Origin::Scan).then_some(kind);
    let exists = std::fs::symlink_metadata(path).is_ok();
    let kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };

    let accessor = self.path_manager.access();
    let is_watched_path = accessor.files().0.contains(path) || accessor.missing().0.contains(path);

    // Drop a removed path's record so it reads as `null`, like watchpack. A
    // directory moved or deleted as a whole is one event with no per-file
    // removals, so its descendants' records go with it. Only a path that can
    // hold records is swept, so deleting many unrelated files stays cheap.
    if removed || kind == FsEventKind::Remove {
      let may_contain_records = self.path_manager.may_contain_records(path);
      self.path_manager.remove_file_time(path);
      if may_contain_records {
        self.path_manager.remove_file_times_under(path);
      }
      if kind == FsEventKind::Remove && is_watched_path {
        self.path_manager.set_reported_file_removal(path);
      }
    }

    // Filter stale FSEvents: on macOS, FSEvents can deliver events for files
    // written before the watcher was created. Stat the file and compare mtime
    // against the recorded baseline to suppress events where nothing changed.
    // Apply the same suppression to Create for already-registered files, since
    // macOS may emit stale Create events for files that predate the watcher.
    // A registered-missing path takes its first record here, once it exists.
    if (kind == FsEventKind::Change || (kind == FsEventKind::Create && is_watched_path))
      && !self.path_manager.has_mtime_changed(path)
      && !matches!(origin, Origin::Scan)
    {
      return;
    }

    if kind != FsEventKind::Remove
      && !self
        .path_manager
        .set_context_entry(path, matches!(origin, Origin::Scan) || is_watched_path)
        .await
    {
      return;
    }
    self.path_manager.set_last_watch_events(path);

    let finder = self.finder();
    let mut events = ResolvedEvents::default();
    self.resolve_event(&finder, &mut events, path, kind, scan_hint);
    if appeared && path.is_dir() && !path.is_symlink() {
      // On Linux, inotify adds a new directory's watch before emitting its
      // Create event, so files written before that watch existed are already
      // on disk; later writes produce their own events. On every backend, a
      // directory moved into place reports only itself. Recover registered
      // descendants here, off the OS reader thread. Duplicate events coalesce
      // in the aggregate sets. Cost is proportional to the new subtree; on
      // Linux it is comparable to notify's own add-watch walk.
      let mut directories = vec![path.clone()];
      while let Some(directory) = directories.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
          continue;
        };
        for entry in entries.flatten() {
          let Ok(file_type) = entry.file_type() else {
            continue;
          };
          let entry_path = InternedPath::from(entry.path());
          let is_registered = finder.contains_path(&entry_path);
          if !file_type.is_dir() && !is_registered {
            continue;
          }
          if self.path_manager.is_ignored_path(entry_path.as_ref()).await {
            continue;
          }

          if is_registered {
            if finder.files.contains(&entry_path)
              && !self.path_manager.has_mtime_changed(&entry_path)
            {
              continue;
            }
            self.resolve_event(&finder, &mut events, &entry_path, FsEventKind::Create, None);
          }
          // DirEntry::file_type does not follow symlinks.
          if file_type.is_dir() {
            directories.push(entry_path);
          }
        }
      }
    }

    let _ = self.tx.send(events);
  }

  /// Helper to construct a `DependencyFinder` for the current path register state.
  fn finder(&self) -> DependencyFinder<'_> {
    let accessor = self.path_manager.access();

    let files = accessor.files().0;
    let directories = accessor.directories().0;
    let missing = accessor.missing().0;

    DependencyFinder {
      files,
      directories,
      missing,
    }
  }

  fn resolve_event(
    &self,
    finder: &DependencyFinder<'_>,
    events: &mut ResolvedEvents,
    path: &InternedPath,
    kind: FsEventKind,
    scan_hint: Option<FsEventKind>,
  ) {
    if scan_hint.is_some() {
      self.analyzer.invalidate_path(path);
    }
    let mut direct = Subscribers::new();
    let mut contexts = SmallVec::<[_; 1]>::new();
    finder.visit_subscriptions(path, |watched, file, directory| {
      let requested = self.path_manager.subscribers(watched, file, directory);
      if requested.is_empty() {
        return;
      }
      if watched == path {
        direct = requested;
        return;
      }
      let removed = std::fs::symlink_metadata(watched).is_err();
      if removed || scan_hint.is_some() {
        self.analyzer.invalidate_path(watched);
      }
      if removed {
        let may_contain_records = self.path_manager.may_contain_records(watched);
        self.path_manager.remove_file_time(watched);
        if may_contain_records {
          self.path_manager.remove_file_times_under(watched);
        }
      }
      contexts.push((watched.clone(), requested, removed));
    });

    let context_removal = kind == FsEventKind::Remove
      && scan_hint != Some(FsEventKind::Remove)
      && contexts.iter().any(|(_, _, removed)| !removed);
    let mut notify_observed = false;
    let mut aliases = RequestedPaths::new();
    for (watched, requested, removed) in contexts {
      for subscriber in requested {
        if subscriber.path == watched {
          notify_observed = true;
        } else if !aliases.contains(&subscriber.path) {
          aliases.push(subscriber.path.clone());
        }
        let kind = if removed && (!context_removal || subscriber.file_or_missing) {
          FsEventKind::Remove
        } else {
          FsEventKind::Change
        };
        events.dependencies.push(FsEvent {
          path: subscriber.path,
          kind,
        });
      }
    }

    // Ordinary nested contexts share one raw child event. Each distinct alias has its own listener.
    let direct_notifies_path = direct.iter().any(|subscriber| subscriber.path == *path);
    let context_notifications =
      aliases.len() + usize::from(notify_observed && (context_removal || !direct_notifies_path));
    if !context_removal {
      events
        .immediate
        .extend((0..context_notifications).map(|_| ImmediateEvent::from_path(path.clone(), kind)));
    }
    for subscriber in direct {
      if !context_removal || subscriber.file_or_missing {
        events
          .immediate
          .push(ImmediateEvent::from_path(subscriber.path.clone(), kind));
        events.dependencies.push(FsEvent {
          path: subscriber.path,
          kind,
        });
      }
    }
    if context_removal {
      events
        .immediate
        .extend((0..context_notifications).map(|_| ImmediateEvent::ContextRemove(path.clone())));
    }
  }
}
#[cfg(test)]
mod tests {

  use std::path::Path;

  use rspack_paths::InternedPath;

  use super::*;

  #[test]
  fn test_find_dependency_for_file() {
    let files = InternedPathDashSet::default();
    let directories = InternedPathDashSet::default();
    let missing = InternedPathDashSet::default();

    let file_0 = InternedPath::from(Path::new("/path/a/b/c/index.js"));
    let dir_0 = InternedPath::from(Path::new("/path/a/b"));
    files.insert(file_0.clone());

    directories.insert(dir_0.clone());
    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events = finder.find_associated_event(&file_0, FsEventKind::Remove);

    assert_eq!(associated_events.len(), 2);

    assert!(associated_events.contains(&(file_0, FsEventKind::Remove)));
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
  }

  #[test]
  fn test_find_dependency_for_directory() {
    let files = InternedPathDashSet::default();
    let directories = InternedPathDashSet::default();
    let missing = InternedPathDashSet::default();

    let dir_0 = InternedPath::from(Path::new("/path/a/b/c"));
    let dir_1 = InternedPath::from(Path::new("/path/a/b"));

    directories.insert(dir_0.clone());
    directories.insert(dir_1.clone());

    let finder = DependencyFinder {
      files: &files,
      directories: &directories,
      missing: &missing,
    };

    let associated_events = finder.find_associated_event(
      &InternedPath::from(Path::new("/path/a/b/c/index.js")),
      FsEventKind::Create,
    );
    assert_eq!(associated_events.len(), 2);
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
    assert!(associated_events.contains(&(dir_1, FsEventKind::Change)));
  }
}
