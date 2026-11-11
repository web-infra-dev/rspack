use std::sync::Arc;

use rspack_paths::{InternedPath, InternedPathDashSet};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  paths::PathManager,
  symlinks::SymlinkAlias,
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
  /// Finds all registered dependencies related to the given path.
  ///
  /// This method checks if the path is a directory or file and then determines if it is registered
  /// in the dependency sets. If it is a directory, it also recursively adds all parent directories
  /// that are registered as directories or missing.
  pub fn find_associated_event(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
  ) -> Vec<(InternedPath, FsEventKind)> {
    let mut paths = Vec::new();

    if path.exists() {
      // If the given path is a directory and is registered as a directory, add it to the result.
      if path.is_dir() && self.contains_directory(path) {
        paths.push((path.clone(), kind));
      }

      // If the given path is a file and is registered as a file, add it to the result.
      if path.is_file() && self.contains_file(path) {
        paths.push((path.clone(), kind));
      }
    } else if self.contains_path(path) {
      // If the path does not exist but is registered as missing, add it to the result.
      paths.push((path.clone(), kind));
    }

    // Recursively add all parent directories that are registered as directories or missing.
    self.recurse_parent_directories(path, &mut paths);

    paths
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

  /// Recursively adds all parent directories that are registered as directories or missing.
  fn recurse_parent_directories(
    &self,
    path: &InternedPath,
    paths: &mut Vec<(InternedPath, FsEventKind)>,
  ) {
    match path.parent() {
      Some(parent) => {
        if self.contains_directory(&InternedPath::from(parent)) {
          // For parent directory, it always FsEventKind::Change its recursive children no matter what kind is
          paths.push((InternedPath::from(parent), FsEventKind::Change));
        }
        self.recurse_parent_directories(&InternedPath::from(parent), paths);
      }
      None => {
        // Reached the root directory, stop recursion
      }
    }
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
enum Origin {
  /// The OS watcher; `appeared` when it created or renamed the path into place.
  Disk {
    appeared: bool,
  },
  /// An explicit event supplied through `FsWatcher::trigger_event`.
  Injected,
  /// A scan, for what changed while the watch was not active yet: a file a
  /// context's initial scan found changed since the start time, or a
  /// registered path the scanner backfills. Already judged a change, it is
  /// never dropped as a duplicate or as attribute-only.
  Scan,
  LinkScan(InternedPath),
  Symlink {
    link_changed: bool,
    appeared: bool,
    injected: bool,
  },
}

impl Origin {
  fn is_scan(&self) -> bool {
    matches!(self, Self::Scan | Self::LinkScan(_))
  }

  fn link_changed(&self) -> bool {
    matches!(
      self,
      Self::LinkScan(_)
        | Self::Symlink {
          link_changed: true,
          ..
        }
    )
  }
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
            if let Origin::LinkScan(source_path) = &origin {
              let source_path = source_path.clone();
              processor
                .process(&path, &source_path, kind, origin, None)
                .await;
              continue;
            }
            if matches!(origin, Origin::Disk { .. })
              && processor.path_manager.is_unchanged_link_event(&path)
            {
              continue;
            }
            let mut mapped_original = false;
            if !origin.is_scan() {
              let include_descendants = kind != FsEventKind::Change
                || matches!(origin, Origin::Disk { appeared: true } | Origin::Injected);
              for SymlinkAlias {
                path: alias,
                link_changed,
                resolved_path,
              } in processor
                .path_manager
                .symlink_aliases(&path, include_descendants)
              {
                if alias != path || link_changed {
                  mapped_original |= alias == path;
                  processor.analyzer.invalidate_path(&alias);
                  processor
                    .process(
                      &alias,
                      &path,
                      kind,
                      Origin::Symlink {
                        link_changed,
                        injected: matches!(origin, Origin::Injected),
                        appeared: matches!(origin, Origin::Disk { appeared: true })
                          || kind == FsEventKind::Create,
                      },
                      resolved_path.as_ref(),
                    )
                    .await;
                }
              }
            }
            if !mapped_original {
              let resolved_path = processor.path_manager.resolved_directory_entry(&path);
              processor
                .process(&path, &path, kind, origin, resolved_path.as_ref())
                .await;
            }
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

  pub(crate) fn on_scanned_link(
    &self,
    path: &InternedPath,
    link: &InternedPath,
    kind: FsEventKind,
  ) {
    let _ = self
      .pending
      .send((path.clone(), kind, Origin::LinkScan(link.clone())));
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
  async fn process(
    &self,
    path: &InternedPath,
    source_path: &InternedPath,
    kind: FsEventKind,
    origin: Origin,
    resolved_path: Option<&InternedPath>,
  ) {
    let appeared = matches!(origin, Origin::Disk { appeared: true })
      || matches!(origin, Origin::Symlink { appeared: true, .. })
      || (matches!(origin, Origin::Injected) && kind == FsEventKind::Create);
    // Drop events inside ignored subtrees. The recursive-root watch delivers
    // events for unregistered paths (e.g. the build-output dir); left through,
    // `find_associated_event` bubbles them to a registered parent and triggers
    // a spurious rebuild.
    // A scan's event was filtered when the scan found the path.
    if !origin.is_scan()
      && self
        .path_manager
        .is_ignored_path(resolved_path.unwrap_or(path))
        .await
    {
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
    let removed = kind == FsEventKind::Remove
      && !matches!(
        origin,
        Origin::Disk { .. }
          | Origin::Symlink {
            injected: false,
            ..
          }
      );
    let initial_missing = removed && origin.is_scan();
    let exists = if origin.link_changed() {
      std::fs::symlink_metadata(source_path).is_ok()
    } else if matches!(origin, Origin::Symlink { .. }) {
      path.exists()
    } else if origin.is_scan() {
      self.path_manager.is_file_present(path)
    } else {
      std::fs::symlink_metadata(path).is_ok()
    };
    let kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };

    let accessor = self.path_manager.access();
    let is_watched_path = accessor.files().0.contains(path) || accessor.missing().0.contains(path);
    let is_followed_context_file = self.path_manager.is_followed_context_file(path);
    if kind == FsEventKind::Remove
      && matches!(
        origin,
        Origin::Symlink {
          link_changed: false,
          ..
        }
      )
      && accessor.missing().0.contains(path)
      && !self.path_manager.has_file_time(path)
    {
      return;
    }
    // Watchpack keeps a discovered file link as an entry when only its target
    // disappears; the link itself is still present in the context.
    if matches!(
      origin,
      Origin::Symlink {
        link_changed: false,
        ..
      }
    ) && !exists
      && !is_watched_path
      && self.path_manager.is_context_symlink_file(path)
    {
      self.path_manager.clear_file_baseline(path);
      return;
    }

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
    if (kind == FsEventKind::Change
      || (kind == FsEventKind::Create && (is_watched_path || is_followed_context_file)))
      && !self.path_manager.has_mtime_changed(path)
      && !origin.is_scan()
      && !origin.link_changed()
    {
      return;
    }

    if kind != FsEventKind::Remove
      && !self
        .path_manager
        .set_context_entry(
          path,
          origin.is_scan() || origin.link_changed() || is_watched_path || is_followed_context_file,
        )
        .await
    {
      return;
    }
    if matches!(origin, Origin::Symlink { .. })
      && kind != FsEventKind::Remove
      && !path.is_dir()
      && (!is_watched_path
        || matches!(
          origin,
          Origin::Symlink {
            link_changed: true,
            ..
          }
        ))
      && let Ok(mtime) = path.metadata().and_then(|metadata| metadata.modified())
    {
      self.path_manager.set_file_time(path, mtime, false, false);
    }
    self.path_manager.set_last_watch_events(path);

    let finder = self.finder();
    let mut associated_event = finder.find_associated_event(path, kind);
    if origin.link_changed()
      && finder.contains_path(path)
      && !associated_event
        .iter()
        .any(|(dependency, _)| dependency == path)
    {
      associated_event.push((path.clone(), kind));
    }
    // A directory above changes with `path`, unless it is gone itself: an
    // event below it may arrive after the directory's own removal.
    // Its records go with it, as for `path` itself.
    for (associated, associated_kind) in &mut associated_event {
      if associated != path
        && *associated_kind == FsEventKind::Change
        && std::fs::symlink_metadata(&*associated).is_err()
      {
        *associated_kind = FsEventKind::Remove;
        self.analyzer.invalidate_path(associated);
        let may_contain_records = self.path_manager.may_contain_records(associated);
        self.path_manager.remove_file_time(associated);
        if may_contain_records {
          self.path_manager.remove_file_times_under(associated);
        }
      }
    }

    let context_removal = (kind == FsEventKind::Remove
      && !initial_missing
      && associated_event
        .iter()
        .any(|(associated, kind)| associated != path && *kind == FsEventKind::Change))
    .then(|| resolved_path.unwrap_or(path).clone());
    let context_only_removal = context_removal.is_some() && !finder.contains_file(path);
    if context_removal.is_some() {
      associated_event.retain_mut(|(associated, associated_kind)| {
        if *associated_kind == FsEventKind::Remove && !finder.contains_file(associated) {
          if associated == path {
            return false;
          }
          *associated_kind = FsEventKind::Change;
        }
        true
      });
    }
    let mut raw = (!associated_event.is_empty() && !context_only_removal).then(|| FsEvent {
      path: path.clone(),
      kind,
    });
    let mut recovered = Vec::new();
    if let (Some(raw), Some(resolved_path)) = (&mut raw, resolved_path) {
      if finder.contains_file(path) {
        recovered.push(FsEvent {
          path: resolved_path.clone(),
          kind,
        });
      } else {
        raw.path = resolved_path.clone();
      }
    }
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
            let associated = finder.find_associated_event(&entry_path, FsEventKind::Create);
            if !associated.is_empty() {
              recovered.push(FsEvent {
                path: entry_path.clone(),
                kind: FsEventKind::Create,
              });
              associated_event.extend(associated);
            }
          }
          // DirEntry::file_type does not follow symlinks.
          if file_type.is_dir() {
            directories.push(entry_path);
          }
        }
      }
    }

    if origin.is_scan() {
      self.analyzer.invalidate_path(path);
      for (associated, _) in &associated_event {
        self.analyzer.invalidate_path(associated);
      }
    }
    self.trigger_events(associated_event, raw, recovered, context_removal);
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

  /// Sends a group of file system events for the given path and event kind.
  /// If the event is successfully sent, it returns true; otherwise, it returns false.
  fn trigger_events(
    &self,
    events: Vec<(InternedPath, FsEventKind)>,
    raw: Option<FsEvent>,
    recovered: EventBatch,
    context_removal: Option<InternedPath>,
  ) -> bool {
    self
      .tx
      .send(ResolvedEvents {
        dependencies: events
          .into_iter()
          .map(|(path, kind)| FsEvent { path, kind })
          .collect(),
        raw,
        recovered,
        context_removal,
      })
      .is_ok()
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
