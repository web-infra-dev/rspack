use std::{
  fmt::Write,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use rspack_util::fx_hash::FxDashMap;

/// The final outcome of a cache lookup, after validation.
#[derive(Debug)]
pub enum CacheRecord {
  /// The cached value passed validation and was reused.
  Hit,
  /// The lookup did not provide a reusable value.
  Miss,
}

/// Final cache decisions for one namespace. Storage reads alone do not imply a hit:
/// callers must first validate and accept the restored value.
#[derive(Debug, Default)]
pub(super) struct CacheCounters {
  hits: AtomicUsize,
  misses: AtomicUsize,
}

impl CacheCounters {
  pub fn record(&self, record: CacheRecord) {
    let counter = match record {
      CacheRecord::Hit => &self.hits,
      CacheRecord::Miss => &self.misses,
    };
    counter.fetch_add(1, Ordering::Relaxed);
  }
}

/// Compiler-scoped counters reset between compilations, independently of shared cache storage.
/// Facades register once and retain their counters for lock-free recording.
#[derive(Debug, Default)]
pub(super) struct CacheStatistics {
  entries: FxDashMap<Arc<str>, Arc<CacheCounters>>,
}

impl CacheStatistics {
  pub fn register(&self, name: Arc<str>) -> Arc<CacheCounters> {
    Arc::clone(self.entries.entry(name).or_default().value())
  }

  pub fn reset(&self) {
    // Reset in place because existing facades retain their counter handles.
    for entry in &self.entries {
      let counters = entry.value();
      counters.hits.store(0, Ordering::Relaxed);
      counters.misses.store(0, Ordering::Relaxed);
    }
  }

  pub fn report(&self, compiler_path: &str, full_hash: Option<&str>) -> Option<String> {
    // All compilation work has joined before taking this snapshot.
    let mut entries = self
      .entries
      .iter()
      .filter_map(|entry| {
        let counters = entry.value();
        let hits = counters.hits.load(Ordering::Relaxed);
        let misses = counters.misses.load(Ordering::Relaxed);
        (hits + misses != 0).then(|| (Arc::clone(entry.key()), hits, misses))
      })
      .collect::<Vec<_>>();
    if entries.is_empty() {
      return None;
    }
    entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut message = String::from("Compilation");
    if let Some(full_hash) = full_hash {
      write!(message, " {full_hash}").expect("writing to a String cannot fail");
    }
    if !compiler_path.is_empty() {
      write!(message, " ({compiler_path})").expect("writing to a String cannot fail");
    }
    let prefix = format!("{compiler_path}|");
    for (name, hits, misses) in entries {
      let name = name.strip_prefix(&prefix).unwrap_or(&name);
      let total = hits + misses;
      let rate = hits as f64 / total as f64 * 100.0;
      write!(message, "\n  {name}: {rate:.1}% ({hits}/{total})")
        .expect("writing to a String cannot fail");
    }
    Some(message)
  }
}
