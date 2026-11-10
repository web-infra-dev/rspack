//! Opt-in diagnostic timings. Sums across concurrent tasks are latency sums,
//! not CPU time or an additive decomposition of whole-build wall time.
use std::{
  fs::{File, OpenOptions},
  io::Write,
  sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicU32, AtomicU64, Ordering},
  },
  time::Instant,
};

static ORIGIN: LazyLock<Instant> = LazyLock::new(Instant::now);
static OUTPUT: LazyLock<Option<Mutex<File>>> = LazyLock::new(|| {
  let path = std::env::var_os("RSPACK_LOADER_TIMING_FILE")?;
  match OpenOptions::new().create(true).append(true).open(&path) {
    Ok(file) => Some(Mutex::new(file)),
    Err(error) => {
      eprintln!("Cannot open RSPACK_LOADER_TIMING_FILE {path:?}: {error}");
      None
    }
  }
});

#[derive(Clone, Copy)]
pub enum TimingPoint {
  Enqueued,
  Received,
  JsEntered,
  ContextStarted,
  ContextPrepared,
  CompleteEntered,
  Returned,
}

pub struct LoaderTaskTiming {
  start: Instant,
  start_ns: u64,
  resource: String,
  loader: String,
  phase: &'static str,
  parallel: bool,
  id: AtomicU32,
  worker: AtomicU32,
  points: [AtomicU64; 7],
}

impl LoaderTaskTiming {
  pub fn new(
    resource: &str,
    loader: &str,
    phase: &'static str,
    parallel: bool,
  ) -> Option<Arc<Self>> {
    OUTPUT.as_ref()?;
    let start_ns = ORIGIN.elapsed().as_nanos() as u64;
    Some(Arc::new(Self {
      start: Instant::now(),
      start_ns,
      resource: resource.to_owned(),
      loader: loader.to_owned(),
      phase,
      parallel,
      id: AtomicU32::new(0),
      worker: AtomicU32::new(0),
      points: std::array::from_fn(|_| AtomicU64::new(0)),
    }))
  }

  pub fn mark(&self, point: TimingPoint) {
    self.points[point as usize].store(
      self.start.elapsed().as_nanos() as u64 + 1,
      Ordering::Relaxed,
    );
  }

  pub fn identify(&self, id: u32, worker: Option<u32>) {
    self.id.store(id, Ordering::Relaxed);
    self.worker.store(worker.unwrap_or(0), Ordering::Relaxed);
  }

  /// Called after the original native state has been restored to the runner.
  /// Logging is outside the recorded interval and enabled only for diagnostics.
  pub fn finish(&self, success: bool) {
    let total_ns = self.start.elapsed().as_nanos() as u64;
    let points: Vec<Option<f64>> = self
      .points
      .iter()
      .map(|point| {
        let value = point.load(Ordering::Relaxed);
        (value != 0).then(|| (value - 1) as f64 / 1_000_000.0)
      })
      .collect();
    let mut record = serde_json::json!({
      "id": self.id.load(Ordering::Relaxed), "threadId": self.worker.load(Ordering::Relaxed),
      "resource": self.resource, "loader": self.loader, "phase": self.phase,
      "parallel": self.parallel, "startNs": self.start_ns.to_string(),
      "pointsMs": points, "totalMs": total_ns as f64 / 1_000_000.0, "success": success,
    })
    .to_string();
    record.push('\n');
    if let Some(output) = OUTPUT.as_ref()
      && let Err(error) = output
        .lock()
        .expect("loader timing log lock poisoned")
        .write_all(record.as_bytes())
    {
      eprintln!("Cannot write RSPACK_LOADER_TIMING_FILE: {error}");
    }
  }
}
