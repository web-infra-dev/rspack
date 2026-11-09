use std::{
  sync::mpsc::{self, Sender},
  time::{Duration, Instant, SystemTime},
};

use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions};

#[derive(Debug)]
enum Observation {
  WriteStarted(&'static str),
  WriteFinished(&'static str),
  Changed(String),
  Aggregate(FxHashSet<String>, FxHashSet<String>),
}

struct Handler(Sender<(Instant, Observation)>);

impl EventHandler for Handler {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((Instant::now(), Observation::Changed(path)));
    Ok(())
  }
}

impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, changed: FxHashSet<String>, deleted: FxHashSet<String>) {
    let _ = self
      .0
      .send((Instant::now(), Observation::Aggregate(changed, deleted)));
  }
}

#[test]
#[ignore = "run explicitly in the Windows burst diagnostics job"]
fn diagnose_burst_aggregation() {
  let last_gap_ms = std::env::var("RSPACK_BURST_LAST_GAP_MS")
    .map(|value| value.parse::<u64>().expect("numeric final write gap"))
    .unwrap_or(150);
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect("create caller runtime");
  let temp = tempfile::tempdir().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let names = ["a", "b", "c", "d", "e", "f", "g"];
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(600),
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
    Box::new(Handler(tx.clone())),
  ));

  let start = Instant::now();
  let (observations, last_write) = std::thread::scope(|scope| {
    let writer = scope.spawn(|| {
      let mut last_write = Instant::now();
      for (index, name) in names.iter().enumerate() {
        if index > 0 {
          let gap_ms = if index == names.len() - 1 {
            last_gap_ms
          } else {
            150
          };
          std::thread::sleep(Duration::from_millis(gap_ms));
        }
        last_write = Instant::now();
        let _ = tx.send((last_write, Observation::WriteStarted(name)));
        std::fs::write(
          base.join(name),
          SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time after epoch")
            .as_millis()
            .to_string(),
        )
        .expect("write fixture");
        let _ = tx.send((Instant::now(), Observation::WriteFinished(name)));
      }
      last_write
    });
    let mut observations = Vec::new();
    let deadline = start + Duration::from_secs(6);
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
      match rx.recv_timeout(remaining.min(Duration::from_millis(1800))) {
        Ok(observation) => observations.push(observation),
        Err(_) => break,
      }
    }
    (observations, writer.join().expect("burst writer panicked"))
  });
  runtime.block_on(watcher.close()).expect("close watcher");

  eprintln!("last_gap_ms={last_gap_ms}");
  for (at, observation) in &observations {
    match observation {
      Observation::WriteStarted(name) => {
        eprintln!("{:?} write-start {name}", *at - start);
      }
      Observation::WriteFinished(name) => {
        eprintln!("{:?} write-end {name}", *at - start);
      }
      Observation::Changed(path) => {
        eprintln!("{:?} change {path}", *at - start);
      }
      Observation::Aggregate(changed, deleted) => {
        eprintln!(
          "{:?} aggregate changed={changed:?} deleted={deleted:?}",
          *at - start
        );
      }
    }
  }
  let batches = observations
    .iter()
    .filter_map(|(at, observation)| match observation {
      Observation::Aggregate(changed, deleted) => Some((at, changed, deleted)),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(batches.len(), if last_gap_ms > 600 { 2 } else { 1 });
  if last_gap_ms > 600 {
    assert_eq!(batches[0].1.len(), names.len() - 1);
    assert_eq!(batches[1].1.len(), 1);
    assert!(
      batches[1]
        .1
        .contains(base.join("g").to_str().expect("UTF-8 fixture"))
    );
  }
  let (at, _, _) = batches.last().expect("at least one aggregate");
  assert!(**at >= last_write + Duration::from_millis(600));
  let mut all_changed = FxHashSet::default();
  for (_, changed, deleted) in batches {
    assert!(deleted.is_empty());
    all_changed.extend(changed.iter().cloned());
  }
  assert_eq!(all_changed.len(), names.len());
  for name in names {
    assert!(all_changed.contains(base.join(name).to_str().expect("UTF-8 fixture")));
  }
}
