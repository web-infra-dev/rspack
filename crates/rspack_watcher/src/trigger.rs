use std::{path::Path, sync::Arc};

use rspack_paths::{InternedPath, InternedPathDashSet};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  paths::{DepKind, EntrySubscriber, PathManager, RebindRequest, disk_mtime},
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

  /// Like [`Self::find_associated_event`], but resolving the logical
  /// subscribers of a *physical* path through the link table first: a disk
  /// event names the physical target, while the caller registered the link.
  ///
  /// Each subscription is judged by its own logical path, so ignoring a target
  /// only silences the contexts that reach it through the ignored spelling.
  pub async fn find_aliased_event(
    &self,
    path: &InternedPath,
    kind_of_event: FsEventKind,
    path_manager: &PathManager,
  ) -> Vec<(InternedPath, FsEventKind)> {
    let mut paths = Vec::new();
    let push = |paths: &mut Vec<(InternedPath, FsEventKind)>, entry: InternedPath, kind| {
      if !paths.iter().any(|(existing, _)| *existing == entry) {
        paths.push((entry, kind));
      }
    };
    for (entry, kind) in self.find_associated_event(path, kind_of_event) {
      // A registered directory link whose physical target is ignored behaves
      // like watchpack's *file* watcher on the link: it reports the link
      // itself, never bubbling its target's changes to the parent context.
      if entry != *path && path_manager.is_unfollowed(&entry) {
        continue;
      }
      // A registered directory link is spelled by its physical target: that is
      // what its own watcher sees, so that is what `ignored` is judged on. The
      // link's logical path may be visible while the entry behind it is not.
      // Without `followSymlinks` there is no such second spelling, and the
      // event's own spelling was already judged by `process`.
      let ignored = if !path_manager.follows_symlinks() {
        false
      } else if entry != *path && path_manager.is_registered_link(&entry) {
        match path_manager.registered_physical_root(&entry) {
          Some(root) => {
            let physical = path.strip_prefix(entry.as_ref() as &Path).map_or_else(
              |_| root.clone(),
              |suffix| InternedPath::from(root.join(suffix)),
            );
            path_manager.is_ignored_path(physical.as_ref()).await
          }
          None => path_manager.is_ignored_path(path.as_ref()).await,
        }
      } else {
        path_manager.is_ignored_path(entry.as_ref()).await
      };
      if !ignored {
        push(&mut paths, entry, kind);
      }
    }
    // Link aliases only exist with `followSymlinks`; without it a link is one
    // plain entry and the event's own spelling is the only subscription.
    if !path_manager.follows_symlinks() {
      return paths;
    }
    for subscriber in path_manager.entry_subscribers(path) {
      let logical = subscriber.logical_path();
      if logical == path {
        continue;
      }
      match &subscriber {
        EntrySubscriber::Registered { kind, .. } => {
          let event_kind = match kind {
            DepKind::Directory => FsEventKind::Change,
            _ => kind_of_event,
          };
          let ign = path_manager.is_ignored_path(logical.as_ref()).await;
          if !ign {
            push(&mut paths, logical.clone(), event_kind);
          }
        }
        // A file link found by a context scan reports only its context, like
        // watchpack's `DirectoryWatcher` for the link: the context is what was
        // registered, and the link path is what its scan found. A target that
        // went away leaves the link itself in place, so nothing changed.
        EntrySubscriber::ContextFile { root_context, .. } => {
          let ign = path_manager.is_ignored_path(root_context.as_ref()).await;
          if kind_of_event != FsEventKind::Remove && !ign {
            push(&mut paths, root_context.clone(), FsEventKind::Change);
          }
        }
      }
    }
    // A context link's target reports the context root it was found under.
    // `ignored` is judged on the subscription's own spelling: a registered
    // directory link is spelled by its physical target, a link a context scan
    // found by its logical path under the context.
    for (root_context, logical, _, registered) in path_manager.context_subscribers(path) {
      let spelled = if registered {
        path.as_ref()
      } else {
        logical.as_ref() as &std::path::Path
      };
      let ign = path_manager.is_ignored_path(spelled).await;
      if !ign {
        push(&mut paths, root_context, FsEventKind::Change);
      }
    }
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

  /// A watched link moved: the logical dependency must be re-resolved and, if
  /// its new target is already on disk, reported as changed.
  pub(crate) fn on_rebind(&self, request: RebindRequest) {
    self.analyzer.invalidate_path(&request.logical_path);
    if request.logical_path.exists() {
      let _ = self
        .pending
        .send((request.logical_path, FsEventKind::Change, Origin::Scan));
    }
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
    //
    // A physical target reached through a link is judged per subscription by
    // its logical path (see `find_aliased_event`): ignoring a target must not
    // hide a link to it that is itself visible, and vice versa. Only without
    // `followSymlinks` is the event's own spelling the sole subscription.
    if !matches!(origin, Origin::Scan) {
      // An event from behind a link whose target is ignored is dropped: the
      // backend still reports through such a link even when only its parent
      // directory is watched.
      if self.path_manager.is_behind_unfollowed_link(path) {
        return;
      }
      if !self.path_manager.follows_symlinks() {
        let reported = self.report_path(path);
        if self.path_manager.is_ignored_path(reported.as_ref()).await {
          return;
        }
      }
    }

    // A watched link was replaced or removed: its logical dependencies must be
    // resolved again, and a new target already on disk reported as changed.
    if !matches!(origin, Origin::Scan)
      && (self.path_manager.tracks_link(path)
        || !self.path_manager.pending_waiters(path).is_empty())
    {
      self.rebind_links(path).await;
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
    let initial_missing = removed && matches!(origin, Origin::Scan);
    let exists = std::fs::symlink_metadata(path).is_ok();
    let kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };

    let accessor = self.path_manager.logical_access();
    // A disk event names the physical target of a followed link; the watched
    // path is the logical dependency it serves.
    let is_watched_path = accessor.files().0.contains(path)
      || accessor.missing().0.contains(path)
      || self
        .path_manager
        .logical_entries(path)
        .iter()
        .any(|logical| {
          accessor.files().0.contains(logical) || accessor.missing().0.contains(logical)
        });

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

    if kind != FsEventKind::Remove {
      let entry_change = self
        .path_manager
        .set_context_entry(path, matches!(origin, Origin::Scan) || is_watched_path)
        .await;
      if entry_change == crate::paths::EntryChange::Unchanged {
        return;
      }
      // A directory that is still the same one: only a subscription to that
      // directory itself needs a *disk* event; the context above it does not.
      // An explicit or scanned event was already declared a change, so it
      // keeps bubbling like any other.
      let existing_directory = entry_change == crate::paths::EntryChange::ExistingDirectory
        && !matches!(origin, Origin::Injected | Origin::Scan);
      if existing_directory {
        let mut associated_event = self
          .finder()
          .find_aliased_event(path, kind, &self.path_manager)
          .await;
        associated_event.retain(|(dependency, _)| dependency == path);
        if !associated_event.is_empty() {
          self.trigger_events(associated_event, Vec::new(), Vec::new(), None);
        }
        return;
      }
    }

    let finder = self.finder();
    let mut associated_event = finder
      .find_aliased_event(path, kind, &self.path_manager)
      .await;
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
    .then(|| self.report_path(path));
    // A removal only a context sees stays a `context_removal`, unless the path
    // is an entry of a registered directory link's physical target: that
    // target's own watcher reports it concretely, like watchpack.
    let context_only_removal = context_removal.is_some()
      && !finder.contains_file(path)
      && !self.path_manager.has_registered_prefix(path);
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
    // The raw (undelayed) event names what the caller registered, once per
    // spelling: watchpack runs one watcher per physical path *and* one per link
    // component, and each emits its own `change`. A file reached through a link
    // found by a context scan is reported under that link's logical path; a link
    // that is itself a registered context keeps the physical target.
    //
    // A removal that only the context sees (the path was never a subscription
    // of its own) stays a `context_removal`: watchpack reports it as a `null`
    // change time, not as a concrete event.
    let raw = if associated_event.is_empty() || context_only_removal {
      Vec::new()
    } else {
      self
        .report_paths(path)
        .await
        .into_iter()
        .map(|raw_path| FsEvent {
          path: raw_path,
          kind,
        })
        .collect()
    };

    let mut recovered = Vec::new();
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

    if matches!(origin, Origin::Scan) {
      self.analyzer.invalidate_path(path);
      for (associated, _) in &associated_event {
        self.analyzer.invalidate_path(associated);
      }
    }
    self.trigger_events(associated_event, raw, recovered, context_removal);
  }
  /// A watched link (or a missing component a dependency waits on) changed on
  /// disk: re-resolve the affected logical dependencies, refresh the physical
  /// watch set, and report the ones whose target is already present.
  async fn rebind_links(&self, path: &InternedPath) {
    let mut requests = self.path_manager.link_waiters(path);
    requests.extend(self.path_manager.pending_waiters(path));
    if requests.is_empty() {
      return;
    }
    let update = self.path_manager.refresh_links().await;
    let mut events = ResolvedEvents::default();
    for request in update.retargeted.into_iter().chain(requests) {
      self.analyzer.invalidate_path(&request.logical_path);
      // A file reached through a link whose target went away changes nothing
      // about the link itself: watchpack stays quiet until the link or its
      // target comes back. A link that moved — or a missing component that
      // appeared — does change what the dependency resolves to.
      let Some(mtime) = disk_mtime(&request.logical_path) else {
        continue;
      };
      self
        .path_manager
        .set_file_time(&request.logical_path, mtime, false, false);
      events.dependencies.push(FsEvent {
        path: request.logical_path,
        kind: FsEventKind::Change,
      });
    }
    if !events.dependencies.is_empty() {
      let _ = self.tx.send(events);
    }
  }

  /// The path an event on `path` is reported under: the logical registration
  /// that reaches it, or `path` itself when nothing aliases it.
  fn report_path(&self, path: &InternedPath) -> InternedPath {
    if self.path_manager.logical_access().files().0.contains(path) {
      return path.clone();
    }
    let mut logical: Vec<InternedPath> = self
      .path_manager
      .entry_subscribers(path)
      .iter()
      .map(|subscriber| subscriber.logical_path().clone())
      .collect();
    for (_, entry, link, registered) in self.path_manager.context_subscribers(path) {
      if !registered {
        logical.push(entry);
      }
      let _ = link;
    }
    logical.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    logical.first().cloned().unwrap_or_else(|| path.clone())
  }

  /// Every spelling of `path` a subscription watches, each of which watchpack
  /// would report a `change` for: the physical path itself, each registered
  /// logical alias, and each context's logical spelling. Each spelling is
  /// judged by `ignored` on its own, so a target hidden behind an ignored
  /// directory is still reported under a visible link.
  async fn report_paths(&self, path: &InternedPath) -> Vec<InternedPath> {
    let mut spelled: Vec<InternedPath> = Vec::new();
    let mut push = |entry: InternedPath| {
      if !spelled.contains(&entry) {
        spelled.push(entry);
      }
    };
    let accessor = self.path_manager.logical_access();
    let follow_symlinks = self.path_manager.follows_symlinks();
    // Without `followSymlinks` a path has exactly one spelling: its own. A link
    // is one plain entry, and a context file is reported as spelled.
    if !follow_symlinks {
      push(path.clone());
    } else {
      let physical_is_watched = accessor.files().0.contains(path)
        || accessor.missing().0.contains(path)
        // An entry of a registered directory link's physical target: that
        // target's own watcher reports it under the physical spelling.
        || self.path_manager.has_registered_prefix(path);
      if physical_is_watched {
        push(path.clone());
      }
    }
    // Link aliases only exist when links are followed; without `followSymlinks`
    // a link is one plain entry and only its own spelling is reported.
    if follow_symlinks {
      // A path spelled logically under a registered directory link is also
      // reported under the physical spelling that link's own watcher uses.
      if let Some(physical) = self.path_manager.physical_spelling(path) {
        push(physical);
      }
      for subscriber in self.path_manager.entry_subscribers(path) {
        push(subscriber.logical_path().clone());
      }
      for (_, entry, link, registered) in self.path_manager.context_subscribers(path) {
        // A registered directory link is spelled physically above; a link a
        // context scan found is spelled by its logical path under the context.
        if !registered {
          push(entry);
        }
        let _ = link;
      }
    }
    // A registered directory link's target is spelled physically by its own
    // watcher: an event on the logical link path is reported under both.
    //
    // Only with `followSymlinks` does a second spelling exist to filter: the
    // event's own spelling was already judged by `process`, and judging it
    // again would call the caller's predicate twice for one path.
    let mut visible = Vec::with_capacity(spelled.len());
    for entry in spelled {
      if !follow_symlinks || !self.path_manager.is_ignored_path(entry.as_ref()).await {
        visible.push(entry);
      }
    }
    // An event no alias explains is still reported as spelled: watchpack emits
    // it from the watcher that saw it.
    if visible.is_empty() && !self.path_manager.is_ignored_path(path.as_ref()).await {
      visible.push(path.clone());
    }
    visible.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    visible
  }

  /// Helper to construct a `DependencyFinder` for the current path register state.
  /// Always over the *logical* registrations: a disk event is mapped back to
  /// the logical path through the link table, never reported as the physical
  /// path the caller never registered.
  fn finder(&self) -> DependencyFinder<'_> {
    let accessor = self.path_manager.logical_access();

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
    raw: EventBatch,
    recovered: EventBatch,
    context_removal: Option<InternedPath>,
  ) -> bool {
    // Every path actually reported to the caller advances the safe time of the
    // contexts above it, which is what `collectTimeInfoEntries` exposes. Doing
    // it here means only *visible* subscriptions count: a context whose link is
    // ignored never hears about the target behind it.
    //
    // A registered directory link's own time moves with its physical target,
    // not with the logical path: its watcher is rooted at the target, so the
    // link's parent context is not involved.
    for (path, _) in &events {
      if self.path_manager.is_registered_link(path) {
        self.path_manager.set_last_watch_event_only(path);
      } else {
        self.path_manager.set_last_watch_events(path);
      }
    }
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
