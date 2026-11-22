use std::sync::Arc;

use rspack_paths::{InternedPath, InternedPathSet};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{
  EventBatch, ResolvedEvents,
  analyzer::{Analyzer, RecommendedAnalyzer},
  paths::PathManager,
  subscription::{LogicalKind, MatchedContext},
};

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
        crate::block_on_current_thread(Box::pin(async move {
          while let Some((path, kind, origin)) = incoming.recv().await {
            processor.process(&path, kind, origin).await;
          }
        }));
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
  async fn check_ignored_cached(
    &self,
    memo: &mut [InternedPathSet; 2],
    path: &InternedPath,
  ) -> bool {
    if memo[0].contains(path) {
      return false;
    }
    if memo[1].contains(path) {
      return true;
    }
    let ignored = self.path_manager.is_ignored_path(path.as_ref()).await;
    memo[usize::from(ignored)].insert(path.clone());
    ignored
  }

  /// Called when a file system event occurs.
  /// Finds all subscribed dependencies related to the given path and triggers events for each.
  async fn process(&self, input_path: &InternedPath, kind: FsEventKind, origin: Origin) {
    let appeared = matches!(origin, Origin::Disk { appeared: true })
      || (matches!(origin, Origin::Injected) && kind == FsEventKind::Create);
    // A scan's event was filtered when the scan found the path.
    let skip_ignore = matches!(origin, Origin::Scan);
    let mut ignored_memo: [InternedPathSet; 2] = Default::default();

    let (mut targets, mut contexts) = self.path_manager.lookup_disk_event(input_path);
    let mut disk_path = input_path.clone();

    if targets.is_empty() && contexts.is_empty() {
      let mapped = [
        LogicalKind::File,
        LogicalKind::Directory,
        LogicalKind::Missing,
      ]
      .into_iter()
      .find_map(|k| {
        let t = self.path_manager.target_path_of(k, input_path);
        (t != *input_path).then_some(t)
      });
      if let Some(target_disk) = mapped {
        disk_path = target_disk;
        let (t, c) = self.path_manager.lookup_disk_event(&disk_path);
        targets = t;
        contexts = c;
      }
    }

    if matches!(origin, Origin::Scan) && disk_path != *input_path {
      targets.retain(|t| t.owner_path == *input_path);
      contexts.retain(|c| c.root_context == *input_path || c.logical_entry_path == *input_path);
    }

    // Drop events inside ignored subtrees. The recursive-root watch delivers
    // events for unregistered paths (e.g. the build-output dir); left through,
    // they would bubble to a covering context and trigger a spurious rebuild.
    if !skip_ignore
      && targets.is_empty()
      && contexts.is_empty()
      && self
        .check_ignored_cached(&mut ignored_memo, input_path)
        .await
    {
      return;
    }

    if !skip_ignore {
      let mut kept_targets = Vec::with_capacity(targets.len());
      for t in targets {
        if !self
          .check_ignored_cached(&mut ignored_memo, &t.owner_path)
          .await
        {
          kept_targets.push(t);
        }
      }
      targets = kept_targets;

      let mut kept_contexts = Vec::with_capacity(contexts.len());
      for c in contexts {
        if !self
          .check_ignored_cached(&mut ignored_memo, &c.logical_entry_path)
          .await
        {
          kept_contexts.push(c);
        }
      }
      contexts = kept_contexts;
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
    let explicit_removed = kind == FsEventKind::Remove && !matches!(origin, Origin::Disk { .. });
    let initial_missing = explicit_removed && matches!(origin, Origin::Scan);
    let exists = std::fs::symlink_metadata(&disk_path).is_ok();
    let effective_kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      other => other,
    };

    // Drop a removed path's record so it reads as `null`, like watchpack. A
    // directory moved or deleted as a whole is one event with no per-file
    // removals, so its descendants' records go with it. Only a path that can
    // hold records is swept, so deleting many unrelated files stays cheap.
    if explicit_removed || effective_kind == FsEventKind::Remove {
      self.path_manager.handle_disk_directory_removed(&disk_path);
      for t in &targets {
        self.path_manager.remove_file_time(&t.owner_path);
        if effective_kind == FsEventKind::Remove {
          self.path_manager.set_reported_file_removal(&t.owner_path);
        }
      }
    }

    let mut dependencies: Vec<(InternedPath, FsEventKind)> = Vec::new();
    let mut raw: Vec<FsEvent> = Vec::new();
    let mut recovered: Vec<FsEvent> = Vec::new();
    let mut context_removals: Vec<InternedPath> = Vec::new();
    let mut judged_logical_paths: InternedPathSet = InternedPathSet::default();

    // 1. Direct target subscriptions (`File` / `Missing`).
    // Filter stale FSEvents: on macOS, FSEvents can deliver events for files
    // written before the watcher was created. Stat the file and compare mtime
    // against the recorded baseline to suppress events where nothing changed.
    // Apply the same suppression to Create for already-registered files, since
    // macOS may emit stale Create events for files that predate the watcher.
    // A registered-missing path takes its first record here, once it exists.
    if !targets.is_empty() {
      let mut any_accepted = false;
      for t in &targets {
        if effective_kind == FsEventKind::Remove {
          any_accepted = true;
          dependencies.push((t.owner_path.clone(), FsEventKind::Remove));
          raw.push(FsEvent {
            path: t.notify_path.clone(),
            kind: FsEventKind::Remove,
          });
        } else {
          let accepted = matches!(origin, Origin::Scan)
            || self
              .path_manager
              .check_and_advance_target_baseline(&t.owner_path, &t.target_path);
          if accepted {
            any_accepted = true;
            judged_logical_paths.insert(t.owner_path.clone());
            self.path_manager.set_last_watch_events(&t.notify_path);
            dependencies.push((t.owner_path.clone(), effective_kind));
            raw.push(FsEvent {
              path: t.notify_path.clone(),
              kind: effective_kind,
            });
          }
        }
      }
      if !any_accepted
        && (effective_kind == FsEventKind::Change || effective_kind == FsEventKind::Create)
        && !matches!(origin, Origin::Scan)
      {
        return;
      }
    }

    // 2. Context directory subscriptions.
    // A directory above changes with `disk_path`, unless it is gone itself: an
    // event below it may arrive after the directory's own removal.
    // Its records go with it, as for `disk_path` itself.
    if effective_kind != FsEventKind::Remove {
      for ctx in &contexts {
        let root_exists = ctx.root_target.is_dir();
        if !root_exists {
          self.analyzer.invalidate_path(&ctx.root_target);
          self.path_manager.mark_directory_absent(&ctx.root_context);
          self.path_manager.remove_file_time(&ctx.root_context);
          self.path_manager.remove_file_times_under(&ctx.root_context);
          dependencies.push((ctx.root_context.clone(), FsEventKind::Remove));
          continue;
        }

        let is_judged = matches!(origin, Origin::Scan)
          || judged_logical_paths.contains(&ctx.logical_entry_path)
          || self
            .path_manager
            .is_registered_file_or_missing(&ctx.logical_entry_path);

        if ctx.is_root_self && self.path_manager.clear_directory_absent(&ctx.root_context) {
          self
            .path_manager
            .recover_context(ctx.root_target.clone(), ctx.root_context.clone())
            .await;
        }

        let accepted = self
          .path_manager
          .set_context_entry_for_binding(&disk_path, ctx, is_judged)
          .await;
        if !accepted {
          continue;
        }
        self
          .path_manager
          .set_last_watch_events(&ctx.logical_entry_path);

        let dep_kind = if ctx.is_root_self {
          effective_kind
        } else {
          FsEventKind::Change
        };
        dependencies.push((ctx.root_context.clone(), dep_kind));
        raw.push(FsEvent {
          path: ctx.logical_entry_path.clone(),
          kind: effective_kind,
        });
      }
    } else if !contexts.is_empty() {
      let mut surviving_outer_changed = false;
      let mut pending_gone_contexts: Vec<&MatchedContext> = Vec::new();

      for ctx in &contexts {
        let root_exists = std::fs::symlink_metadata(&ctx.root_target).is_ok()
          && (!ctx.is_root_self || ctx.root_target.exists());
        if root_exists && !ctx.is_root_self {
          surviving_outer_changed = true;
          let may_contain = self
            .path_manager
            .may_contain_records(&ctx.logical_entry_path);
          self.path_manager.remove_file_time(&ctx.logical_entry_path);
          if may_contain {
            self
              .path_manager
              .remove_file_times_under(&ctx.logical_entry_path);
          }
          self
            .path_manager
            .set_last_watch_events(&ctx.logical_entry_path);
          dependencies.push((ctx.root_context.clone(), FsEventKind::Change));
          if !initial_missing {
            context_removals.push(ctx.logical_entry_path.clone());
          }
        } else {
          pending_gone_contexts.push(ctx);
        }
      }

      for ctx in pending_gone_contexts {
        self.analyzer.invalidate_path(&ctx.root_target);
        self.path_manager.mark_directory_absent(&ctx.root_context);
        self.path_manager.remove_file_time(&ctx.root_context);
        self.path_manager.remove_file_times_under(&ctx.root_context);
        self
          .path_manager
          .handle_disk_directory_removed(&ctx.root_target);

        if surviving_outer_changed && !initial_missing {
          if !ctx.is_root_self {
            dependencies.push((ctx.root_context.clone(), FsEventKind::Change));
          }
        } else {
          dependencies.push((ctx.root_context.clone(), FsEventKind::Remove));
          raw.push(FsEvent {
            path: ctx.logical_entry_path.clone(),
            kind: FsEventKind::Remove,
          });
        }
      }
    }

    // 3. If a directory appeared, check registered subtree targets and absent contexts inside it.
    // On Linux, inotify adds a new directory's watch before emitting its
    // Create event, so files written before that watch existed are already
    // on disk; later writes produce their own events. On every backend, a
    // directory moved into place reports only itself. Recover registered
    // descendants here, off the OS reader thread. Duplicate events coalesce
    // in the aggregate sets.
    if appeared && disk_path.is_dir() && !disk_path.is_symlink() {
      for ctx_dir in self
        .path_manager
        .recover_absent_contexts_under(&disk_path)
        .await
      {
        if !skip_ignore && self.check_ignored_cached(&mut ignored_memo, &ctx_dir).await {
          continue;
        }
        dependencies.push((ctx_dir.clone(), FsEventKind::Change));
        recovered.push(FsEvent {
          path: ctx_dir,
          kind: FsEventKind::Change,
        });
      }

      let subtree_targets = self.path_manager.collect_subtree_targets(&disk_path);
      for sub_target in subtree_targets {
        if sub_target.target_path == disk_path {
          continue;
        }
        if std::fs::symlink_metadata(&sub_target.target_path).is_err() {
          continue;
        }
        if !skip_ignore
          && self
            .check_ignored_cached(&mut ignored_memo, &sub_target.owner_path)
            .await
        {
          continue;
        }
        if self
          .path_manager
          .check_and_advance_target_baseline(&sub_target.owner_path, &sub_target.target_path)
        {
          self
            .path_manager
            .set_last_watch_events(&sub_target.notify_path);
          dependencies.push((sub_target.owner_path.clone(), FsEventKind::Create));
          recovered.push(FsEvent {
            path: sub_target.notify_path.clone(),
            kind: FsEventKind::Create,
          });
          let (_, covering_contexts) = self.path_manager.lookup_disk_event(&sub_target.target_path);
          for ctx in covering_contexts {
            if !ctx.is_root_self && ctx.root_target.is_dir() {
              let _ = self
                .path_manager
                .set_context_entry_for_binding(&sub_target.target_path, &ctx, true)
                .await;
              self
                .path_manager
                .set_last_watch_events(&ctx.logical_entry_path);
              dependencies.push((ctx.root_context.clone(), FsEventKind::Change));
            }
          }
        }
      }
    }

    if matches!(origin, Origin::Scan) {
      self.analyzer.invalidate_path(&disk_path);
      for (associated, _) in &dependencies {
        let target = self
          .path_manager
          .target_path_of(LogicalKind::Directory, associated);
        self.analyzer.invalidate_path(&target);
        self.analyzer.invalidate_path(associated);
      }
    }

    self.dispatch_deduped(dependencies, raw, recovered, context_removals);
  }

  #[inline(never)]
  fn dispatch_deduped(
    &self,
    dependencies: Vec<(InternedPath, FsEventKind)>,
    raw: Vec<FsEvent>,
    recovered: Vec<FsEvent>,
    context_removals: Vec<InternedPath>,
  ) {
    if dependencies.is_empty()
      && raw.is_empty()
      && recovered.is_empty()
      && context_removals.is_empty()
    {
      return;
    }

    let mut seen_deps: [InternedPathSet; 2] = Default::default();
    let deduped_deps: EventBatch = dependencies
      .into_iter()
      .filter(|(p, k)| seen_deps[usize::from(*k == FsEventKind::Remove)].insert(p.clone()))
      .map(|(path, kind)| FsEvent { path, kind })
      .collect();

    let mut seen_raw: [InternedPathSet; 2] = Default::default();
    let deduped_raw: EventBatch = raw
      .into_iter()
      .filter(|ev| seen_raw[usize::from(ev.kind == FsEventKind::Remove)].insert(ev.path.clone()))
      .collect();

    let deduped_recovered: EventBatch = recovered
      .into_iter()
      .filter(|ev| seen_raw[usize::from(ev.kind == FsEventKind::Remove)].insert(ev.path.clone()))
      .collect();

    let mut seen_ctx_rem = InternedPathSet::default();
    let deduped_ctx_removals: Vec<InternedPath> = context_removals
      .into_iter()
      .filter(|p| seen_ctx_rem.insert(p.clone()))
      .collect();

    let _ = self.tx.send(ResolvedEvents {
      dependencies: deduped_deps,
      raw: deduped_raw,
      recovered: deduped_recovered,
      context_removals: deduped_ctx_removals,
    });
  }
}
