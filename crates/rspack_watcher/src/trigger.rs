use std::sync::Arc;

use rspack_paths::InternedPath;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use super::{FsEvent, FsEventKind};
use crate::{EventBatch, WatcherManager, state::Origin};

/// `Trigger` is responsible for sending file system events to the event channel
/// when a relevant file or directory change is detected.
///
/// It only buffers: the OS watcher callback hands an event over and returns
/// immediately, and a dedicated thread drains the buffer one event at a time.
pub struct Trigger {
  pending: UnboundedSender<(InternedPath, FsEventKind, Origin)>,
}

impl Trigger {
  /// Create a new `Trigger` with the given watch state and event sender.
  ///
  /// Each event is classified by [`WatcherManager::process`], deliberately NOT on
  /// the thread that reads events from the OS. One event costs several `stat`
  /// calls, and a JS-backed `ignored` predicate costs a full Node event loop
  /// turn; doing that inline stalls the reader, and on Linux the inotify queue
  /// is bounded — a stalled reader means the kernel silently drops events
  /// (`IN_Q_OVERFLOW`).
  pub fn new(state: Arc<WatcherManager>, tx: UnboundedSender<EventBatch>) -> Self {
    let (pending, mut incoming) = unbounded_channel::<(InternedPath, FsEventKind, Origin)>();

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
            if let Some(events) = state.process(&path, kind, origin).await {
              trigger_events(&tx, events);
            }
          }
        });
      })
      .expect("spawn watcher event filter thread");

    Self { pending }
  }

  /// Hands a raw event to the filter thread. Never blocks — this runs straight
  /// from the OS watcher callback.
  pub fn on_event(&self, path: &InternedPath, kind: FsEventKind) {
    self.on_disk_event(path, kind, kind == FsEventKind::Create);
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

/// Sends a group of file system events to the watcher executor.
fn trigger_events(tx: &UnboundedSender<EventBatch>, events: Vec<(InternedPath, FsEventKind)>) {
  let _ = tx.send(
    events
      .into_iter()
      .map(|(path, kind)| FsEvent { path, kind })
      .collect(),
  );
}
