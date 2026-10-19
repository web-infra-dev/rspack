use std::time::{Instant, SystemTime};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;

use crate::{EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions};

struct Aggregate;

impl EventAggregateHandler for Aggregate {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}

  fn on_error(&self, error: rspack_error::Error) {
    panic!("watch failed: {error}");
  }
}

struct Change;
impl EventHandler for Change {}

fn benchmark_watch_delta(single_file: bool, scattered: bool) {
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect("create runtime");
  runtime.block_on(async {
    for missing_count in [0, 1_000, 10_000] {
      let dir = tempfile::tempdir().expect("create directory");
      let context = InternedPath::from(dir.path().canonicalize().expect("canonicalize directory"));
      let files = [context.join("first.js"), context.join("second.js")].map(InternedPath::from);
      if single_file {
        for file in &files {
          std::fs::write(file, "entry").expect("write file");
        }
      }
      let missing = (0..missing_count)
        .map(|index| InternedPath::from(context.join(if scattered {
          format!("missing-dir-{index}/entry.js")
        } else {
          format!("missing-{index}.js")
        })))
        .collect::<Vec<_>>();
      let watcher = FsWatcher::new(FsWatcherOptions::default(), Default::default());
      watcher
        .watch(
          (std::iter::empty(), std::iter::empty()),
          (std::iter::once(context), std::iter::empty()),
          (missing.into_iter(), std::iter::empty()),
          SystemTime::now(),
          Box::new(Aggregate),
          Box::new(Change),
        )
        .await;
      let mut samples = Vec::with_capacity(200);
      for iteration in 0..220 {
        if iteration == 20 {
          eprintln!("WATCHER_FS_SAMPLE_BEGIN single_file={single_file} scattered={scattered} missing={missing_count}");
        }
        let added = single_file.then(|| files[iteration % 2].clone());
        let removed = (single_file && iteration > 0).then(|| files[(iteration - 1) % 2].clone());
        let start = Instant::now();
        watcher
          .watch(
            (added.into_iter(), removed.into_iter()),
            (std::iter::empty(), std::iter::empty()),
            (std::iter::empty(), std::iter::empty()),
            SystemTime::now(),
            Box::new(Aggregate),
            Box::new(Change),
          )
          .await;
        if iteration >= 20 {
          samples.push(start.elapsed().as_micros());
        }
      }
      eprintln!("WATCHER_FS_SAMPLE_END");
      watcher.close().await.expect("close watcher");
      samples.sort_unstable();
      println!(
        "single_file={single_file}, scattered={scattered}, missing={missing_count}, n={}, p50_us={}, p95_us={}, mean_us={}",
        samples.len(),
        samples[100],
        samples[190],
        samples.iter().sum::<u128>() / samples.len() as u128,
      );
    }
  });
}

#[test]
#[ignore = "manual steady-state watch benchmark"]
fn benchmark_empty_watch_delta() {
  benchmark_watch_delta(false, false);
}

#[test]
#[ignore = "manual single-file subscription benchmark"]
fn benchmark_single_file_watch_delta() {
  benchmark_watch_delta(true, false);
}

#[test]
#[ignore = "manual scattered missing subscription benchmark"]
fn benchmark_scattered_missing_watch_delta() {
  benchmark_watch_delta(false, true);
  benchmark_watch_delta(true, true);
}
