use std::{
  sync::mpsc::{self, Sender},
  time::{Duration, Instant, SystemTime},
};

use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherOptions,
  testing::{ExecutorEvent, take_executor_events},
};

const TIMEOUT: Duration = Duration::from_millis(600);

struct Handler(Sender<(FxHashSet<String>, FxHashSet<String>)>);

impl EventHandler for Handler {}

impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, changed: FxHashSet<String>, deleted: FxHashSet<String>) {
    let _ = self.0.send((changed, deleted));
  }
}

fn run_burst(last_gap: Duration) {
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect("create caller runtime");
  let temp = tempfile::tempdir().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let names = ["a", "b", "c", "d", "e", "f", "g"];
  let expected_files = names
    .iter()
    .map(|name| base.join(name).to_string_lossy().into_owned())
    .collect::<FxHashSet<_>>();
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(TIMEOUT.as_millis() as u32),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, rx) = mpsc::channel();
  runtime.block_on(watcher.watch(
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    (
      names.iter().map(|name| base.join(name).into()),
      std::iter::empty(),
    ),
    SystemTime::now(),
    Box::new(Handler(tx.clone())),
    Box::new(Handler(tx)),
  ));

  let start = Instant::now();
  let (delivered, writes) = std::thread::scope(|scope| {
    let writer = scope.spawn(|| {
      let mut writes = Vec::new();
      for (index, name) in names.iter().enumerate() {
        if index > 0 {
          std::thread::sleep(if index == names.len() - 1 {
            last_gap
          } else {
            Duration::from_millis(150)
          });
        }
        let began = Instant::now();
        std::fs::write(base.join(name), "changed").expect("write fixture");
        writes.push((*name, began, Instant::now()));
      }
      writes
    });
    let deadline = start + Duration::from_secs(6);
    let mut delivered = Vec::new();
    let mut reported = FxHashSet::default();
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
      let complete = writer.is_finished() && reported == expected_files;
      let wait = if complete {
        remaining.min(TIMEOUT * 3)
      } else {
        remaining
      };
      match rx.recv_timeout(wait) {
        Ok((changed, deleted)) => {
          reported.extend(changed.iter().cloned());
          delivered.push((changed, deleted));
        }
        Err(_) => break,
      }
    }
    (delivered, writer.join().expect("burst writer panicked"))
  });
  runtime.block_on(watcher.close()).expect("close watcher");
  let timeline = take_executor_events(&watcher);
  for (name, began, ended) in &writes {
    eprintln!("write {name}: {:?}..{:?}", *began - start, *ended - start);
  }
  for event in &timeline {
    match event {
      ExecutorEvent::Accepted { at, events } => {
        eprintln!("{:?} accepted {events:?}", *at - start);
      }
      ExecutorEvent::Aggregated {
        at,
        changed_files,
        deleted_files,
      } => {
        eprintln!(
          "{:?} aggregate changed={changed_files:?} deleted={deleted_files:?}",
          *at - start
        );
      }
    }
  }
  let recorded_batches = assert_legal_batches(&timeline);
  let delivered_batches = delivered
    .into_iter()
    .map(|(changed, deleted)| {
      assert!(deleted.is_empty());
      changed
    })
    .collect::<Vec<_>>();
  assert_eq!(
    delivered_batches, recorded_batches,
    "aggregate delivery differs from the snapshot"
  );
  let reported = delivered_batches
    .into_iter()
    .flatten()
    .collect::<FxHashSet<_>>();
  assert_eq!(
    reported, expected_files,
    "written files were not all reported"
  );
  for event in timeline {
    if let ExecutorEvent::Accepted { at, events } = event {
      for (path, _) in events {
        let (_, began, _) = writes
          .iter()
          .find(|(name, _, _)| base.join(name).to_str() == Some(path.as_str()))
          .expect("accepted event names a written file");
        assert!(at >= *began, "event was accepted before the write started");
      }
    }
  }
}

#[test]
fn should_aggregate_a_burst_after_the_last_change() {
  run_burst(Duration::from_millis(150));
}

#[test]
fn should_aggregate_a_burst_with_a_long_final_gap() {
  run_burst(Duration::from_millis(900));
}

fn assert_legal_batches(timeline: &[ExecutorEvent]) -> Vec<FxHashSet<String>> {
  let mut pending = FxHashSet::default();
  let mut last_accepted = None;
  let mut batches = Vec::new();
  for event in timeline {
    match event {
      ExecutorEvent::Accepted { at, events } => {
        for (path, kind) in events {
          assert!(matches!(kind, FsEventKind::Change | FsEventKind::Create));
          pending.insert(path.clone());
        }
        last_accepted = Some(*at);
      }
      ExecutorEvent::Aggregated {
        at,
        changed_files,
        deleted_files,
      } => {
        assert!(!pending.is_empty(), "empty aggregate");
        assert!(deleted_files.is_empty());
        assert_eq!(
          changed_files, &pending,
          "aggregate paths differ from the pending accepted events"
        );
        assert!(
          at.duration_since(last_accepted.expect("aggregate needs an accepted event")) >= TIMEOUT,
          "aggregate arrived before the last accepted event's timeout"
        );
        pending.clear();
        batches.push(changed_files.clone());
      }
    }
  }
  assert!(pending.is_empty(), "accepted events were never aggregated");
  batches
}

fn accepted(at: Instant, files: &[&str]) -> ExecutorEvent {
  ExecutorEvent::Accepted {
    at,
    events: files
      .iter()
      .map(|file| (String::from(*file), FsEventKind::Change))
      .collect(),
  }
}

fn aggregated(at: Instant, files: &[&str]) -> ExecutorEvent {
  ExecutorEvent::Aggregated {
    at,
    changed_files: files.iter().map(|file| String::from(*file)).collect(),
    deleted_files: Default::default(),
  }
}

#[test]
fn accepts_a_split_after_a_long_gap() {
  let start = Instant::now();
  let timeline = [
    accepted(start, &["a"]),
    accepted(start + Duration::from_millis(150), &["b"]),
    aggregated(start + Duration::from_millis(750), &["a", "b"]),
    accepted(start + Duration::from_millis(1050), &["c"]),
    aggregated(start + Duration::from_millis(1650), &["c"]),
  ];
  assert_eq!(assert_legal_batches(&timeline).len(), 2);
}

#[test]
fn accepts_a_late_timer_merging_events_across_a_long_gap() {
  let start = Instant::now();
  let timeline = [
    accepted(start, &["a"]),
    accepted(start + Duration::from_millis(150), &["b"]),
    accepted(start + Duration::from_millis(1050), &["c"]),
    aggregated(start + Duration::from_millis(1650), &["a", "b", "c"]),
  ];
  assert_eq!(assert_legal_batches(&timeline).len(), 1);
}

#[test]
#[should_panic(expected = "aggregate arrived before the last accepted event's timeout")]
fn rejects_a_timer_running_from_the_first_event() {
  let start = Instant::now();
  assert_legal_batches(&[
    accepted(start, &["a"]),
    accepted(start + Duration::from_millis(150), &["b"]),
    aggregated(start + TIMEOUT, &["a", "b"]),
  ]);
}

#[test]
#[should_panic(expected = "aggregate paths differ from the pending accepted events")]
fn rejects_a_batch_missing_an_accepted_file() {
  let start = Instant::now();
  assert_legal_batches(&[
    accepted(start, &["a", "b"]),
    aggregated(start + TIMEOUT, &["a"]),
  ]);
}

#[test]
#[should_panic(expected = "accepted events were never aggregated")]
fn rejects_an_event_without_an_aggregate() {
  assert_legal_batches(&[accepted(Instant::now(), &["a"])]);
}
