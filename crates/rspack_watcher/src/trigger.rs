use std::{fs::FileType, sync::Arc};

use rspack_paths::InternedPath;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ImmediateEvent, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  paths::{PathManager, followed_metadata, metadata_if_present, modification_time},
  subscriptions::{ContextSubscribers, SubscriberPaths},
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

struct ObservedChange {
  kind: FsEventKind,
  target_type: Option<FileType>,
  directory_removed: bool,
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
  async fn process(&self, path: &InternedPath, hint: FsEventKind, origin: Origin) {
    if !matches!(origin, Origin::Scan) && self.path_manager.is_ignored_path(path).await {
      return;
    }
    let children = match origin {
      Origin::Disk { appeared } => {
        self
          .path_manager
          .check_registered_children(path, appeared)
          .await
      }
      Origin::Injected
        if hint == FsEventKind::Create
          && path.symlink_metadata().is_ok_and(|entry| entry.is_dir()) =>
      {
        self
          .path_manager
          .check_registered_children(path, true)
          .await
      }
      _ => EventBatch::new(),
    };
    let change = self.record_change_if_fresh(path, hint, origin).await;
    if change.is_none() && children.is_empty() {
      return;
    }
    let mut events = ResolvedEvents::default();
    if let Some(change) = change {
      self.append_subscriber_notifications(
        &mut events,
        path,
        &change,
        matches!(origin, Origin::Scan).then_some(hint),
      );
    }
    for child in children {
      self.analyzer.invalidate_path(&child.path);
      if let Some(change) = self
        .record_change_if_fresh(&child.path, child.kind, Origin::Check)
        .await
      {
        self.append_subscriber_notifications(&mut events, &child.path, &change, None);
      }
    }
    let _ = self.tx.send(events);
  }

  async fn record_change_if_fresh(
    &self,
    path: &InternedPath,
    hint: FsEventKind,
    origin: Origin,
  ) -> Option<ObservedChange> {
    let scanned = matches!(origin, Origin::Scan);
    let entry = metadata_if_present(path.symlink_metadata()).ok()?;
    let target = followed_metadata(path, entry.as_ref()).ok().flatten();
    let file_dependency = self.path_manager.has_file_or_missing_subscription(path);
    let directory_removed = target.as_deref().is_some_and(|target| !target.is_dir())
      && self.path_manager.was_directory(path);
    if entry.is_none()
      && file_dependency
      && matches!(origin, Origin::Disk { .. })
      && self.path_manager.is_known_absent(path)
    {
      return None;
    }
    // An explicit or scanned removal invalidates the baseline even if the path is back.
    if directory_removed
      || entry.is_none()
      || (hint == FsEventKind::Remove && matches!(origin, Origin::Injected | Origin::Scan))
    {
      self.path_manager.remove_path_records(path);
    }

    let kind = if let Some(entry) = &entry {
      // Scans already found a change. They still advance the file's baseline.
      if file_dependency
        && !self
          .path_manager
          .observe_file_mtime(path, target.as_deref().and_then(modification_time))
        && !scanned
        && !directory_removed
      {
        return None;
      }
      if !self
        .path_manager
        .observe_context_entry(path, entry, scanned || file_dependency)
        .await
      {
        return None;
      }
      // An unlink followed by a recreate can deliver a late removal.
      if hint == FsEventKind::Remove {
        FsEventKind::Change
      } else {
        hint
      }
    } else {
      if file_dependency {
        self.path_manager.set_reported_file_removal(path);
      }
      FsEventKind::Remove
    };
    self.path_manager.set_last_watch_events(path);
    Some(ObservedChange {
      kind,
      target_type: target.as_deref().map(std::fs::Metadata::file_type),
      directory_removed,
    })
  }

  fn append_subscriber_notifications(
    &self,
    events: &mut ResolvedEvents,
    path: &InternedPath,
    change: &ObservedChange,
    scanned_kind: Option<FsEventKind>,
  ) {
    let kind = change.kind;
    if scanned_kind.is_some() {
      self.analyzer.invalidate_path(path);
    }
    let (direct, contexts) = self.path_manager.subscriptions.subscribers(
      path,
      change.target_type,
      change.directory_removed,
    );
    let removed_contexts = self.removed_contexts(&contexts, kind, scanned_kind.is_some());
    let context_removal = kind == FsEventKind::Remove
      && scanned_kind != Some(FsEventKind::Remove)
      && removed_contexts.len() < contexts.len();

    let direct_notifies_path = direct
      .iter()
      .any(|s| s.path == *path && (!change.directory_removed || s.file_or_missing));
    let mut notify_observed_path = false;
    let mut aliases = SubscriberPaths::new();
    for context in contexts {
      let removed = removed_contexts.contains(&context.watched);
      for subscriber in context.requested {
        if subscriber.path == context.watched {
          notify_observed_path = true;
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
    // Nested ordinary contexts share the raw child event. Each alias has its own listener.
    let context_notifications = aliases.len()
      + usize::from(notify_observed_path && (context_removal || !direct_notifies_path));
    if !context_removal {
      events
        .immediate
        .extend((0..context_notifications).map(|_| ImmediateEvent::from_path(path.clone(), kind)));
    }
    for subscriber in direct {
      if !context_removal || subscriber.file_or_missing {
        let kind = if change.directory_removed && !subscriber.file_or_missing {
          FsEventKind::Remove
        } else {
          kind
        };
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
      if kind == FsEventKind::Remove
        && matches!(metadata_if_present(path.symlink_metadata()), Ok(None))
      {
        self.analyzer.invalidate_path(path);
        self.path_manager.remove_path_records(path);
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
    let (direct, contexts) = subscriptions.subscribers(
      path,
      std::fs::metadata(path).ok().map(|m| m.file_type()),
      false,
    );
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
