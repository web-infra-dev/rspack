use std::sync::Arc;

use rspack_paths::InternedPath;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ImmediateEvent, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  paths::{PathManager, followed_metadata, modification_time},
  subscriptions::ContextSubscribers,
};
/// Classifies one raw file system event: drops what the `ignored` filter
/// rejects, normalizes the kind against the disk, and resolves the paths to notify.
///
/// Deliberately NOT run on the thread that reads events from the OS. One event
/// costs several `stat` calls, and a JS-backed `ignored` predicate costs a full
/// Node event loop turn; doing that inline stalls the reader, and on Linux the
/// inotify queue is bounded — a stalled reader means the kernel silently drops
/// events (`IN_Q_OVERFLOW`).
struct EventProcessor {
  /// Shared reference to the path register, which tracks watched files/directories/missing.
  path_manager: Arc<PathManager>,
  tx: UnboundedSender<ResolvedEvents>,
  analyzer: Arc<RecommendedAnalyzer>,
}

/// `Trigger` is responsible for sending file system events to the event channel
/// when a relevant file or directory change is detected.
///
/// It only buffers: the OS watcher callback hands an event over and returns
/// immediately, and a dedicated thread drains the buffer one event at a time.
pub struct Trigger {
  pending: UnboundedSender<Pending>,
  pub(crate) analyzer: Arc<RecommendedAnalyzer>,
}

enum Pending {
  Event(InternedPath, FsEventKind, Origin),
  Check(tokio::sync::oneshot::Sender<()>),
}

/// Where an event comes from.
#[derive(Clone, Copy)]
enum Origin {
  /// The OS watcher; `appeared` when it created or renamed the path into place.
  Disk { appeared: bool },
  /// A difference found by checking the current subscriptions.
  Check,
  /// An explicit event supplied through `FsWatcher::trigger_event`.
  Injected,
  /// A scan, for what changed while the watch was not active yet: a file a
  /// context's initial scan found changed since the start time, or a
  /// registered path the scanner backfills. Already judged a change, it is
  /// never dropped as a duplicate or as attribute-only.
  Scan,
}

impl Trigger {
  pub fn new(path_manager: Arc<PathManager>, tx: UnboundedSender<ResolvedEvents>) -> Self {
    let (pending, mut incoming) = unbounded_channel::<Pending>();
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
          while let Some(work) = incoming.recv().await {
            match work {
              Pending::Event(path, kind, origin) => processor.process(&path, kind, origin).await,
              Pending::Check(done) => {
                for event in processor.path_manager.check_subscriptions().await {
                  processor
                    .process(&event.path, event.kind, Origin::Check)
                    .await;
                }
                let _ = done.send(());
              }
            }
          }
        });
      })
      .expect("spawn watcher event filter thread");

    Self { pending, analyzer }
  }

  pub(crate) fn check_subscriptions(&self) -> tokio::sync::oneshot::Receiver<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = self.pending.send(Pending::Check(tx));
    rx
  }

  /// Queues an explicit event from `FsWatcher::trigger_event` without blocking.
  pub fn on_event(&self, path: &InternedPath, kind: FsEventKind) {
    self.analyzer.invalidate_path(path);
    let _ = self
      .pending
      .send(Pending::Event(path.clone(), kind, Origin::Injected));
  }

  /// Preserves whether an OS event created or renamed a path into place,
  /// without treating ordinary directory changes as appearances.
  pub(crate) fn on_disk_event(&self, path: &InternedPath, kind: FsEventKind, appeared: bool) {
    let _ = self.pending.send(Pending::Event(
      path.clone(),
      kind,
      Origin::Disk { appeared },
    ));
  }

  /// A context's initial scan found `path` changed since the start time, while
  /// the watch was not active yet: like watchpack's initial scan, it is
  /// reported as found, never taken for an attribute-only event.
  pub(crate) fn on_scanned_change(&self, path: &InternedPath) {
    let _ = self.pending.send(Pending::Event(
      path.clone(),
      FsEventKind::Change,
      Origin::Scan,
    ));
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
            let _ = pending.send(Pending::Event(event.path, event.kind, Origin::Scan));
          }
        }
      })
      .expect("spawn watcher scan backfill thread");
    tx
  }
}

impl EventProcessor {
  async fn process(&self, path: &InternedPath, kind: FsEventKind, origin: Origin) {
    let appeared = matches!(origin, Origin::Disk { appeared: true })
      || (matches!(origin, Origin::Injected) && kind == FsEventKind::Create);
    // Drop events inside ignored subtrees. The recursive-root watch delivers
    // events for unregistered paths (e.g. the build-output dir); left through,
    // they reach a registered parent and trigger
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
    let removed = kind == FsEventKind::Remove && matches!(origin, Origin::Injected | Origin::Scan);
    let scanned_kind = matches!(origin, Origin::Scan).then_some(kind);
    let own = std::fs::symlink_metadata(path).ok();
    let target = own
      .as_ref()
      .and_then(|entry| followed_metadata(path, entry));
    let exists = own.is_some();
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
    if is_watched_path
      && matches!(kind, FsEventKind::Change | FsEventKind::Create)
      && !self
        .path_manager
        .observe_file_mtime(path, target.as_deref().and_then(modification_time))
      && !matches!(origin, Origin::Scan)
    {
      return;
    }

    if kind != FsEventKind::Remove
      && !self
        .path_manager
        .set_context_entry(
          path,
          matches!(origin, Origin::Scan) || is_watched_path,
          own.as_ref(),
        )
        .await
    {
      return;
    }
    self.path_manager.set_last_watch_events(path);

    let mut events = ResolvedEvents::default();
    self.resolve_into(&mut events, path, kind, target.as_deref(), scanned_kind);
    let directory_hint =
      appeared || (matches!(origin, Origin::Disk { .. }) && kind == FsEventKind::Change);
    if directory_hint && own.as_ref().is_some_and(std::fs::Metadata::is_dir) {
      // A directory moved into place can have just one OS event. Follow only
      // registered child names, even when their on-disk mtimes are old.
      let mut pending = self.path_manager.subscriptions.registered_children(path);
      while let Some(child) = pending.pop() {
        let Ok(metadata) = std::fs::symlink_metadata(&child) else {
          continue;
        };
        if self.path_manager.is_ignored_path(child.as_ref()).await {
          continue;
        }
        if metadata.is_dir() {
          pending.extend(self.path_manager.subscriptions.registered_children(&child));
        }
        let file = accessor.files().0.contains(&child) || accessor.missing().0.contains(&child);
        if !(file || accessor.directories().0.contains(&child)) {
          continue;
        }
        let target = followed_metadata(&child, &metadata);
        let mtime = target.as_deref().and_then(modification_time);
        let recovered_kind = if appeared {
          FsEventKind::Create
        } else {
          if !file || mtime.is_none() {
            continue;
          }
          let Some(kind) = self.path_manager.file_difference(&child, mtime) else {
            continue;
          };
          kind
        };
        if file && !self.path_manager.observe_file_mtime(&child, mtime) {
          continue;
        }
        self.resolve_into(&mut events, &child, recovered_kind, target.as_deref(), None);
      }
    }
    let _ = self.tx.send(events);
  }

  fn resolve_into(
    &self,
    events: &mut ResolvedEvents,
    path: &InternedPath,
    kind: FsEventKind,
    target: Option<&std::fs::Metadata>,
    scanned_kind: Option<FsEventKind>,
  ) {
    if scanned_kind.is_some() {
      self.analyzer.invalidate_path(path);
    }
    let (direct, contexts) = self.path_manager.subscriptions.subscribers(path, target);
    let removed_contexts = self.removed_contexts(&contexts, kind, scanned_kind.is_some());
    let context_removal = kind == FsEventKind::Remove
      && scanned_kind != Some(FsEventKind::Remove)
      && removed_contexts.len() < contexts.len();

    let direct_notifies_path = direct.iter().any(|subscriber| subscriber.path == *path);
    let mut context_removals = 0;
    for context in contexts {
      let removed = removed_contexts.contains(&context.watched);
      for subscriber in context.requested {
        if context_removal {
          context_removals += 1;
        } else if !direct_notifies_path {
          events
            .immediate
            .push(ImmediateEvent::from_path(path.clone(), kind));
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
    events
      .immediate
      .extend((0..context_removals).map(|_| ImmediateEvent::ContextRemove(path.clone())));
  }

  fn removed_contexts(
    &self,
    contexts: &[ContextSubscribers],
    kind: FsEventKind,
    scanned: bool,
  ) -> smallvec::SmallVec<[InternedPath; 1]> {
    let mut removed = smallvec::SmallVec::new();
    if !scanned && kind != FsEventKind::Remove {
      return removed;
    }
    for context in contexts {
      let path = &context.watched;
      if kind == FsEventKind::Remove && std::fs::symlink_metadata(path).is_err() {
        self.analyzer.invalidate_path(path);
        let descendants = self.path_manager.may_contain_records(path);
        self.path_manager.remove_file_time(path);
        if descendants {
          self.path_manager.remove_file_times_under(path);
        }
        removed.push(path.clone());
      } else if scanned {
        self.analyzer.invalidate_path(path);
      }
    }
    removed
  }
}

// The original lookup fixtures construct the three registration sets directly.
#[cfg(test)]
use rspack_paths::InternedPathDashSet;

#[cfg(test)]
struct DependencyFinder<'a> {
  files: &'a InternedPathDashSet,
  directories: &'a InternedPathDashSet,
  missing: &'a InternedPathDashSet,
}

#[cfg(test)]
impl DependencyFinder<'_> {
  fn find_associated_event(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
  ) -> Vec<(InternedPath, FsEventKind)> {
    use crate::subscriptions::{DirectorySubscriptions, Subscription};
    let subscriptions = DirectorySubscriptions::default();
    for f in self.files.iter() {
      subscriptions.update(&f, Subscription::File(f.clone()), true);
    }
    for d in self.directories.iter() {
      subscriptions.update(&d, Subscription::Directory(d.clone()), true);
    }
    for m in self.missing.iter() {
      subscriptions.update(&m, Subscription::Missing(m.clone()), true);
    }
    let (direct, contexts) = subscriptions.subscribers(path, std::fs::metadata(path).ok().as_ref());
    direct
      .into_iter()
      .map(|s| (s.path, kind))
      .chain(
        contexts
          .into_iter()
          .map(|context| (context.watched, FsEventKind::Change)),
      )
      .collect()
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
