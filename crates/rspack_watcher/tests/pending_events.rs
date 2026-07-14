use std::time::SystemTime;

use rspack_paths::ArcPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherOptions,
};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

type Aggregate = (FxHashSet<String>, FxHashSet<String>, u32);

struct AggregateHandler(UnboundedSender<Aggregate>);

impl EventAggregateHandler for AggregateHandler {
  fn on_event_handle(&self, _changed: FxHashSet<String>, _removed: FxHashSet<String>) {
    unreachable!("the executor should provide an aggregate generation");
  }

  fn on_event_handle_with_generation(
    &self,
    changed: FxHashSet<String>,
    removed: FxHashSet<String>,
    generation: u32,
  ) -> bool {
    let _ = self.0.send((changed, removed, generation));
    true
  }
}

struct ChangeHandler(UnboundedSender<(FsEventKind, String)>);

impl EventHandler for ChangeHandler {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((FsEventKind::Change, path));
    Ok(())
  }

  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((FsEventKind::Remove, path));
    Ok(())
  }
}

fn empty_paths() -> (std::iter::Empty<ArcPath>, std::iter::Empty<ArcPath>) {
  (std::iter::empty(), std::iter::empty())
}

fn path_string(path: &ArcPath) -> String {
  path.as_ref().to_string_lossy().into_owned()
}

async fn watch(
  watcher: &mut FsWatcher,
  paths: &[ArcPath],
  aggregate_tx: &UnboundedSender<Aggregate>,
  change_tx: &UnboundedSender<(FsEventKind, String)>,
) {
  watcher
    .watch(
      empty_paths(),
      empty_paths(),
      (paths.iter().cloned(), std::iter::empty()),
      SystemTime::now(),
      Box::new(AggregateHandler(aggregate_tx.clone())),
      Box::new(ChangeHandler(change_tx.clone())),
    )
    .await;
}

async fn receive_changes(
  receiver: &mut UnboundedReceiver<(FsEventKind, String)>,
  count: usize,
) -> Vec<(FsEventKind, String)> {
  let mut events = Vec::with_capacity(count);
  for _ in 0..count {
    let event = tokio::time::timeout(std::time::Duration::from_secs(10), receiver.recv())
      .await
      .expect("watcher event should arrive")
      .expect("watcher event channel should stay open");
    events.push(event);
  }
  events
}

async fn receive_aggregate(receiver: &mut UnboundedReceiver<Aggregate>) -> Aggregate {
  tokio::time::timeout(std::time::Duration::from_secs(10), receiver.recv())
    .await
    .expect("aggregate should arrive")
    .expect("aggregate channel should stay open")
}

#[tokio::test]
async fn pending_events_are_consumed_once_without_aggregate_replay() {
  let temp_dir = TempDir::new().expect("temporary directory should be created");
  let changed_path = ArcPath::from(temp_dir.path().join("changed.js"));
  let removed_path = ArcPath::from(temp_dir.path().join("removed.js"));

  let mut watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(200),
      ..Default::default()
    },
    Default::default(),
  );
  let (aggregate_tx, mut aggregate_rx) = unbounded_channel();
  let (change_tx, mut change_rx) = unbounded_channel();
  let paths = [changed_path.clone(), removed_path.clone()];

  watch(&mut watcher, &paths, &aggregate_tx, &change_tx).await;
  watcher.trigger_event(&changed_path, FsEventKind::Create);
  watcher.trigger_event(&removed_path, FsEventKind::Remove);
  assert_eq!(
    receive_changes(&mut change_rx, 2).await,
    [
      (FsEventKind::Change, path_string(&changed_path)),
      (FsEventKind::Remove, path_string(&removed_path)),
    ],
  );
  watcher.pause().expect("watcher should pause");

  let (changes, removals, generation) = watcher.take_pending_events();
  assert_eq!(changes, FxHashSet::from_iter([path_string(&changed_path)]));
  assert_eq!(removals, FxHashSet::from_iter([path_string(&removed_path)]));
  watcher.trigger_event(&removed_path, FsEventKind::Create);
  assert_eq!(
    receive_changes(&mut change_rx, 1).await,
    [(FsEventKind::Change, path_string(&removed_path))],
  );
  let (changes, removals, next_generation) = watcher.take_pending_events();
  assert_eq!(changes, FxHashSet::from_iter([path_string(&removed_path)]));
  assert!(removals.is_empty());
  assert!(next_generation > generation);
  assert!(
    tokio::time::timeout(std::time::Duration::from_millis(500), aggregate_rx.recv())
      .await
      .is_err(),
    "a scheduled aggregate must not claim events after pause",
  );

  watch(&mut watcher, &paths, &aggregate_tx, &change_tx).await;
  assert!(
    tokio::time::timeout(std::time::Duration::from_millis(500), aggregate_rx.recv())
      .await
      .is_err(),
    "consumed events must not be replayed by the aggregate handler",
  );

  watcher.trigger_event(&changed_path, FsEventKind::Create);
  watcher.trigger_event(&changed_path, FsEventKind::Remove);
  watcher.trigger_event(&changed_path, FsEventKind::Create);
  assert_eq!(
    receive_changes(&mut change_rx, 3).await,
    [
      (FsEventKind::Change, path_string(&changed_path)),
      (FsEventKind::Remove, path_string(&changed_path)),
      (FsEventKind::Change, path_string(&changed_path)),
    ],
  );

  let (changes, removals, aggregate_generation) = receive_aggregate(&mut aggregate_rx).await;
  assert_eq!(changes, FxHashSet::from_iter([path_string(&changed_path)]));
  assert!(removals.is_empty());
  watcher.acknowledge_pending_events(aggregate_generation);

  let (changes, removals, drained_generation) = watcher.take_pending_events();
  assert!(changes.is_empty());
  assert!(removals.is_empty());
  assert!(drained_generation > aggregate_generation);

  watch(&mut watcher, &paths, &aggregate_tx, &change_tx).await;
  watcher.trigger_event(&changed_path, FsEventKind::Create);
  assert_eq!(
    receive_changes(&mut change_rx, 1).await,
    [(FsEventKind::Change, path_string(&changed_path))],
  );
  let (changes, removals, aggregate_generation) = receive_aggregate(&mut aggregate_rx).await;
  assert_eq!(changes, FxHashSet::from_iter([path_string(&changed_path)]));
  assert!(removals.is_empty());

  watcher.trigger_event(&changed_path, FsEventKind::Remove);
  watcher.trigger_event(&removed_path, FsEventKind::Create);
  assert_eq!(
    receive_changes(&mut change_rx, 2).await,
    [
      (FsEventKind::Remove, path_string(&changed_path)),
      (FsEventKind::Change, path_string(&removed_path)),
    ],
  );

  // The aggregate handler queues delivery asynchronously. Until JS acknowledges
  // it, a synchronous drain must recover that claimed batch and supersede the
  // late callback generation.
  let (changes, removals, drained_generation) = watcher.take_pending_events();
  assert_eq!(changes, FxHashSet::from_iter([path_string(&removed_path)]));
  assert_eq!(removals, FxHashSet::from_iter([path_string(&changed_path)]));
  assert!(drained_generation > aggregate_generation);
  watcher.acknowledge_pending_events(aggregate_generation);
  assert!(
    tokio::time::timeout(std::time::Duration::from_millis(500), aggregate_rx.recv())
      .await
      .is_err(),
    "coalesced events should produce one aggregate",
  );

  watcher.close().await.expect("watcher should close");
}
