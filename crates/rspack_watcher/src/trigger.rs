use std::{
  collections::VecDeque,
  sync::{Arc, Mutex, Weak},
};

use rspack_paths::{InternedPath, InternedPathDashSet};
use tokio::sync::{
  mpsc::{UnboundedSender, unbounded_channel},
  oneshot,
};

use super::{FsEvent, FsEventKind};
#[cfg(test)]
use crate::watcher_manager::DirectorySubscriptions;
use crate::{
  EventBatch, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  executor::{Delivery, WatchCycle},
  observation::Mutation,
  paths::{Backfill, PathManager, ScanSink},
  paused::BufferedEvent,
  watcher_manager::{ContextChange, ContextLookup, WatcherManager},
};

#[cfg(test)]
impl ContextLookup for InternedPathDashSet {
  fn subscriptions(&self, path: &InternedPath) -> DirectorySubscriptions {
    DirectorySubscriptions {
      explicit: self.contains(path),
      parent: None,
    }
  }
}

struct DependencyFinder<'a, D = WatcherManager> {
  files: &'a InternedPathDashSet,
  directories: &'a D,
  missing: &'a InternedPathDashSet,
}

impl<D: ContextLookup> DependencyFinder<'_, D> {
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
    self.find_matching(
      path,
      kind,
      |p| self.contains_file(p),
      |p, explicit| explicit || self.missing.contains(p),
    )
  }

  fn find_matching(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
    watches_file: impl Fn(&InternedPath) -> bool,
    watches_directory: impl Fn(&InternedPath, bool) -> bool,
  ) -> Vec<(InternedPath, FsEventKind)> {
    let directory = || watches_directory(path, self.directories.subscriptions(path).explicit);
    let watched = match path.metadata() {
      Ok(metadata) if metadata.is_dir() => directory(),
      Ok(metadata) => metadata.is_file() && watches_file(path),
      Err(_) => watches_file(path) || directory(),
    };
    let mut paths = Vec::new();
    if watched {
      paths.push((path.clone(), kind));
    }
    self.collect_parent_dependencies(path, &mut paths, watches_directory);
    paths
  }

  /// Checks if the given path is registered as a file or missing.
  fn contains_file(&self, path: &InternedPath) -> bool {
    self.files.contains(path) || self.missing.contains(path)
  }

  fn contains_path(&self, path: &InternedPath) -> bool {
    self.contains_file(path) || self.directories.subscriptions(path).explicit
  }

  fn collect_parent_dependencies(
    &self,
    path: &InternedPath,
    paths: &mut Vec<(InternedPath, FsEventKind)>,
    watches_directory: impl Fn(&InternedPath, bool) -> bool,
  ) {
    let mut parent = path.parent().map(InternedPath::from);
    while let Some(directory) = parent {
      let subscriptions = self.directories.subscriptions(&directory);
      // An event can arrive before a new directory was scanned or after its members were swept.
      parent = subscriptions
        .parent
        .or_else(|| directory.parent().map(InternedPath::from));
      if watches_directory(&directory, subscriptions.explicit) {
        paths.push((directory, FsEventKind::Change));
      }
    }
  }
}

fn raw_event(
  path: &InternedPath,
  kind: FsEventKind,
  initial_missing: bool,
  associated_event: &mut Vec<(InternedPath, FsEventKind)>,
  is_file: impl Fn(&InternedPath) -> bool,
) -> (Option<FsEvent>, Option<InternedPath>) {
  let context_removal = (kind == FsEventKind::Remove
    && !initial_missing
    && associated_event
      .iter()
      .any(|(associated, kind)| associated != path && *kind == FsEventKind::Change))
  .then(|| path.clone());
  let context_only_removal = context_removal.is_some() && !is_file(path);
  if context_removal.is_some() {
    associated_event.retain_mut(|(associated, associated_kind)| {
      if *associated_kind == FsEventKind::Remove && !is_file(associated) {
        if associated == path {
          return false;
        }
        *associated_kind = FsEventKind::Change;
      }
      true
    });
  }
  let raw = (!associated_event.is_empty() && !context_only_removal).then(|| FsEvent {
    path: path.clone(),
    kind,
  });
  (raw, context_removal)
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
  output: Output,
  analyzer: Arc<RecommendedAnalyzer>,
  scan_sink: ScanSink,
}

/// `Trigger` is responsible for sending file system events to the event channel
/// when a relevant file or directory change is detected.
///
/// It only buffers: the OS watcher callback hands an event over and returns
/// immediately, and a dedicated thread drains the buffer one event at a time.
pub struct Trigger {
  queue: EventQueue,
  pub(super) scan_sink: ScanSink,
  pub(crate) analyzer: Arc<RecommendedAnalyzer>,
}

/// Where an event comes from.
enum Origin {
  /// The OS watcher; `appeared` when it created or renamed the path into place.
  Disk { appeared: bool },
  /// An explicit event supplied through `FsWatcher::trigger_event`.
  Injected,
  /// A scan, for what changed while the watch was not active yet: a file a
  /// context's initial scan found changed since the start time, or a
  /// registered path the scanner backfills. Already judged a change, it is
  /// never dropped as a duplicate or as attribute-only.
  Scan(Option<Backfill>),
}

pub(crate) struct Input {
  path: InternedPath,
  kind: FsEventKind,
  origin: Origin,
}

enum Output {
  Watch(Arc<Delivery>),
  #[cfg(any(test, feature = "testing"))]
  Channel(UnboundedSender<ResolvedEvents>),
}

enum Work {
  Input(Input),
  Update(oneshot::Receiver<WatchCycle>),
  Notify(InternedPath, ContextChange),
}

type Rechecks = Vec<(InternedPath, ContextChange)>;

#[derive(Clone)]
struct EventQueue {
  tx: UnboundedSender<Work>,
  // While an earlier input waits on JS, new discoveries may already have
  // changed on disk. Finish that input with a fresh check, before delivery.
  after_input: Arc<Mutex<VecDeque<Rechecks>>>,
}

impl EventQueue {
  fn push(&self, path: InternedPath, kind: FsEventKind, origin: Origin) {
    let mut waiting = self.after_input.lock().expect("watch input queue poisoned");
    if self
      .tx
      .send(Work::Input(Input { path, kind, origin }))
      .is_ok()
    {
      waiting.push_back(Vec::new());
    }
  }

  fn scan_sink(&self) -> ScanSink {
    let tx = self.tx.downgrade();
    let after_input = Arc::clone(&self.after_input);
    Arc::new(move |path, change| {
      let Some(tx) = tx.upgrade() else {
        return;
      };
      let mut waiting = after_input.lock().expect("watch input queue poisoned");
      if let Some(checks) = waiting.back_mut() {
        checks.push((path.clone(), change.clone()));
      }
      let _ = tx.send(Work::Notify(path, change));
    })
  }
}

impl Trigger {
  #[cfg(any(test, feature = "testing"))]
  pub(crate) fn new(path_manager: Arc<PathManager>, tx: UnboundedSender<ResolvedEvents>) -> Self {
    Self::with_output(path_manager, Output::Channel(tx))
  }

  pub(crate) fn with_delivery(paths: Arc<PathManager>, delivery: Arc<Delivery>) -> Self {
    Self::with_output(paths, Output::Watch(delivery))
  }

  /// Queue the subscription boundary without waiting on an in-flight JS ignored predicate.
  pub(crate) fn begin_watch(&self) -> oneshot::Sender<WatchCycle> {
    let (resume, ready) = oneshot::channel();
    let _ = self.queue.tx.send(Work::Update(ready));
    resume
  }

  fn with_output(path_manager: Arc<PathManager>, output: Output) -> Self {
    let (tx_work, mut incoming) = unbounded_channel();
    let queue = EventQueue {
      tx: tx_work,
      after_input: Arc::default(),
    };
    let scan_sink = queue.scan_sink();
    let after_input = Arc::clone(&queue.after_input);
    let analyzer = Arc::default();
    let processor = EventProcessor {
      scan_sink: Arc::clone(&scan_sink),
      path_manager,
      output,
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
          .max_blocking_threads(crate::paths::DirectoryScan::MAX_METADATA_READS)
          .enable_all()
          .build()
          .expect("create watcher event filter runtime");

        runtime.block_on(async move {
          while let Some(work) = incoming.recv().await {
            match work {
              Work::Update(ready) => {
                if let Ok(cycle) = ready.await
                  && let Output::Watch(delivery) = &processor.output
                  && delivery.finish_watch(cycle, &processor.path_manager)
                {
                  processor.replay(delivery, cycle).await;
                }
              }
              Work::Input(input) => {
                let events = processor.process(&input, false, &[]).await;
                processor.publish(input, events);
                let checks = after_input
                  .lock()
                  .expect("watch input queue poisoned")
                  .pop_front()
                  .expect("queued input");
                for (path, change) in checks {
                  if let Some(entry) = processor
                    .path_manager
                    .recheck_scanned_entry(&path, change.entry)
                    .await
                  {
                    change.pending.rechecked(entry);
                  }
                }
              }
              Work::Notify(path, change) => processor.deliver_scan(&path, change),
            }
          }
        });
      })
      .expect("spawn watcher event filter thread");

    Self {
      queue,
      analyzer,
      scan_sink,
    }
  }

  /// Queues an explicit event from `FsWatcher::trigger_event` without blocking.
  pub fn on_event(&self, path: &InternedPath, kind: FsEventKind) {
    self.analyzer.invalidate_path(path);
    self.queue.push(path.clone(), kind, Origin::Injected);
  }

  /// Preserves whether an OS event created or renamed a path into place,
  /// without treating ordinary directory changes as appearances.
  pub(crate) fn on_disk_event(&self, path: &InternedPath, kind: FsEventKind, appeared: bool) {
    self
      .queue
      .push(path.clone(), kind, Origin::Disk { appeared });
  }

  #[cfg(feature = "testing")]
  pub(crate) fn on_scanned_change(&self, path: &InternedPath) {
    self
      .queue
      .push(path.clone(), FsEventKind::Change, Origin::Scan(None));
  }

  pub(crate) fn backfill_sink(&self) -> crate::scanner::BackfillSink {
    let queue = self.queue.clone();
    Arc::new(move |event, backfill| {
      queue.push(event.path, event.kind, Origin::Scan(Some(backfill)))
    })
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
    input: &Input,
    detected: bool,
    previous_recoveries: &[FsEvent],
  ) -> Option<ResolvedEvents> {
    let kind = input.kind;
    let origin = &input.origin;
    let path = &input.path;
    let backfill = match origin {
      Origin::Scan(backfill) => backfill.as_ref(),
      _ => None,
    };
    if backfill.is_some_and(|b| b.current_recipients(&self.path_manager).next().is_none()) {
      return None;
    }
    let scanned = matches!(origin, Origin::Scan(_));
    let appeared = matches!(origin, Origin::Disk { appeared: true })
      || (matches!(origin, Origin::Injected) && kind == FsEventKind::Create);
    // Drop events inside ignored subtrees. The recursive-root watch delivers
    // events for unregistered paths (e.g. the build-output dir); left through,
    // `find_associated_event` bubbles them to a registered parent and triggers
    // a spurious rebuild.
    // A scan's event was filtered when the scan found the path.
    if !scanned && self.path_manager.is_ignored_path(path.as_ref()).await {
      return None;
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
    let disk = matches!(origin, Origin::Disk { .. });
    let removed = kind == FsEventKind::Remove && !disk;
    let pending = self.path_manager.watchers.pending_change(path);
    let metadata = if disk {
      let Ok(observation) = self.path_manager.read_disk_entry(path, true) else {
        return None;
      };
      observation
    } else {
      std::fs::symlink_metadata(path).ok()
    };
    let exists = metadata.is_some();
    let kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };

    let finder = self.finder();
    let watches_file = |p: &InternedPath| self.path_manager.watches_file(p, backfill);

    // Drop a removed path's record so it reads as `null`, like watchpack. A
    // directory moved or deleted as a whole is one event with no per-file
    // removals, so its descendants' records go with it. Only a path that can
    // hold records is swept, so deleting many unrelated files stays cheap.
    if !disk && (removed || kind == FsEventKind::Remove) {
      self
        .path_manager
        .set_missing(path, kind == FsEventKind::Remove, backfill);
    }

    // Filter stale FSEvents: on macOS, FSEvents can deliver events for files
    // written before the watcher was created. Stat the file and compare mtime
    // against the recorded baseline to suppress events where nothing changed.
    // Apply the same suppression to Create for already-registered files, since
    // macOS may emit stale Create events for files that predate the watcher.
    // A registered-missing path takes its first record here, once it exists.
    let watched_file = watches_file(path);
    if kind != FsEventKind::Remove
      && watched_file
      && !self.path_manager.has_mtime_changed_for(path, backfill)
      && !scanned
      && !detected
    {
      return None;
    }

    let already_recorded = match backfill {
      Some(b) => b
        .current_recipients(&self.path_manager)
        .any(|s| s.as_ref() == path),
      None => scanned || watched_file,
    };
    if kind != FsEventKind::Remove
      && !self
        .path_manager
        .set_context_entry(path, already_recorded, Some(&self.scan_sink))
        .await
      && !scanned
      && !detected
    {
      return None;
    }
    self.path_manager.observe_live(path, || {
      self.path_manager.set_last_watch_events(path);
      let mutation = match metadata.as_ref() {
        Some(m) if m.is_file() => Mutation::Subtree,
        // A link can be an explicit context before its parent indexes the link itself.
        Some(m) if m.is_symlink() => Mutation::Metadata,
        _ => Mutation::Entry,
      };
      ((), mutation)
    });

    let mut associated_event = match backfill {
      Some(b) => finder.find_matching(path, kind, watches_file, |p, _| {
        b.watches_directory(&self.path_manager, p)
      }),
      None => finder.find_associated_event(path, kind),
    };
    if backfill.is_some() && associated_event.is_empty() {
      return None;
    }
    // A directory above changes with `path`, unless it is gone itself: an
    // event below it may arrive after the directory's own removal.
    // Its records go with it, as for `path` itself.
    for (associated, associated_kind) in &mut associated_event {
      if associated != path
        && *associated_kind == FsEventKind::Change
        && matches!(
          self.path_manager.read_disk_entry(associated, false),
          Ok(None)
        )
      {
        *associated_kind = FsEventKind::Remove;
        self.analyzer.invalidate_path(associated);
      }
    }

    let initial_missing = removed && scanned && associated_event.iter().any(|(dep, _)| dep == path);
    let (raw, context_removal) = raw_event(
      path,
      kind,
      initial_missing,
      &mut associated_event,
      watches_file,
    );
    let mut recovered = Vec::new();
    if appeared
      && self.path_manager.has_registered_descendants(path)
      && path.is_dir()
      && !path.is_symlink()
    {
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
              && !previous_recoveries.iter().any(|e| e.path == entry_path)
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

    let _state = self.path_manager.watchers.lock_observations();
    if let Some(pending) = pending {
      pending.acknowledge(&associated_event);
    }
    if let Some(pending) = self.path_manager.watchers.pending_change(path) {
      pending.acknowledge(&associated_event);
    }
    if scanned {
      self.analyzer.invalidate_path(path);
      for (associated, _) in &associated_event {
        self.analyzer.invalidate_path(associated);
      }
    }
    Some(Self::resolved(
      associated_event,
      raw,
      recovered,
      context_removal,
    ))
  }

  fn deliver_scan(&self, path: &InternedPath, change: ContextChange) {
    let _state = self.path_manager.watchers.lock_observations();
    let (subscribers, restored_mtime, entry) = change.pending.take_subscribers();
    if subscribers.is_empty() {
      return;
    }
    let kind = entry.kind();
    let mut deps = Vec::with_capacity(subscribers.len());
    for subscriber in &subscribers {
      let dep = subscriber.as_ref();
      if let Some(kind) = self.path_manager.scan_event_for(path, subscriber, entry)
        && !deps.iter().any(|(p, _)| p == dep)
      {
        deps.push((dep.clone(), kind));
      }
    }
    if deps.is_empty() {
      return;
    }
    if deps.iter().any(|(dep, _)| dep == path) {
      self
        .path_manager
        .report_scan_delivery(path, &subscribers, restored_mtime, entry);
    }
    self.path_manager.set_last_watch_events(path);
    self.analyzer.invalidate_path(path);
    for (dep, _) in &deps {
      self.analyzer.invalidate_path(dep);
    }
    let (raw, context_removal) = raw_event(path, kind, false, &mut deps, |dep| {
      subscribers
        .iter()
        .any(|sub| sub.as_ref() == dep && self.path_manager.is_scanned_file_recipient(sub))
    });
    self.send(
      None,
      Some(Self::resolved(deps, raw, Vec::new(), context_removal)),
      subscribers.iter().map(Arc::downgrade).collect(),
    );
  }

  /// Helper to construct a `DependencyFinder` for the current path register state.
  fn finder(&self) -> DependencyFinder<'_> {
    let accessor = self.path_manager.access();

    let files = accessor.files().0;
    let missing = accessor.missing().0;

    DependencyFinder {
      files,
      directories: &self.path_manager.watchers,
      missing,
    }
  }

  fn resolved(
    events: Vec<(InternedPath, FsEventKind)>,
    raw: Option<FsEvent>,
    recovered: EventBatch,
    context_removal: Option<InternedPath>,
  ) -> ResolvedEvents {
    ResolvedEvents {
      dependencies: events
        .into_iter()
        .map(|(path, kind)| FsEvent { path, kind })
        .collect(),
      raw,
      recovered,
      context_removal,
    }
  }

  fn publish(&self, input: Input, events: Option<ResolvedEvents>) {
    match input.origin {
      Origin::Scan(Some(backfill)) => self.send(None, events, backfill.recipients),
      Origin::Scan(None) => self.send(None, events, Vec::new()),
      _ => self.send(Some(input), events, Vec::new()),
    }
  }

  fn send(
    &self,
    input: Option<Input>,
    events: Option<ResolvedEvents>,
    subscribers: Vec<Weak<InternedPath>>,
  ) {
    match &self.output {
      Output::Watch(delivery) => delivery.publish(input, events, subscribers),
      #[cfg(any(test, feature = "testing"))]
      Output::Channel(tx) => {
        if let Some(events) = events {
          let _ = tx.send(events);
        }
      }
    }
  }

  async fn replay(&self, delivery: &Delivery, cycle: WatchCycle) {
    while let Some(event) = delivery.next_replay(cycle) {
      let events = match event {
        BufferedEvent::Input {
          input,
          detected,
          recovered,
        } => self.process(&input, detected, &recovered).await,
        BufferedEvent::Scan {
          events,
          subscribers,
        } => {
          let mut events = (*events).clone();
          events.retain_dependencies(|event| {
            subscribers
              .iter()
              .filter_map(|s| s.upgrade())
              .any(|s| *s == event.path && self.path_manager.current_subscriber(&s))
          });
          Some(events)
        }
      };
      delivery.complete_replay(cycle, events);
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
