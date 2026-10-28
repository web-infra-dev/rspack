use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashMap;

/// Cache event counts for one namespace.
#[derive(Debug)]
pub struct CacheInfo {
  pub hits: usize,
  pub misses: usize,
  pub reused: usize,
  pub stores: usize,
}

/// A cache lookup, reuse, or write request.
#[derive(Debug)]
pub enum CacheRecord {
  /// The lookup returned a cached value.
  Hit,
  /// The lookup did not return a cached value.
  Miss,
  /// The caller accepted and reused the cached value.
  Reuse,
  /// A cache write was requested.
  Store,
}

/// Cache event counts for one namespace. A hit does not imply reuse:
/// callers may reject a cached value after validation.
#[derive(Debug, Default)]
pub struct CacheCounters {
  hits: AtomicUsize,
  misses: AtomicUsize,
  reused: AtomicUsize,
  stores: AtomicUsize,
}

impl CacheCounters {
  pub fn record(&self, record: CacheRecord) {
    let counter = match record {
      CacheRecord::Hit => &self.hits,
      CacheRecord::Miss => &self.misses,
      CacheRecord::Reuse => &self.reused,
      CacheRecord::Store => &self.stores,
    };
    counter.fetch_add(1, Ordering::Relaxed);
  }
}

/// Compiler-scoped counters reset between compilations, independently of shared cache storage.
/// Facades register once and retain their counters for lock-free recording.
#[derive(Debug, Default)]
pub struct CacheStatistics {
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
      counters.reused.store(0, Ordering::Relaxed);
      counters.stores.store(0, Ordering::Relaxed);
    }
  }

  pub fn snapshot(&self) -> FxHashMap<String, CacheInfo> {
    self
      .entries
      .iter()
      .filter_map(|entry| {
        let counters = entry.value();
        let hits = counters.hits.load(Ordering::Relaxed);
        let misses = counters.misses.load(Ordering::Relaxed);
        let reused = counters.reused.load(Ordering::Relaxed);
        let stores = counters.stores.load(Ordering::Relaxed);
        (hits != 0 || misses != 0 || reused != 0 || stores != 0).then(|| {
          (
            entry.key().to_string(),
            CacheInfo {
              hits,
              misses,
              reused,
              stores,
            },
          )
        })
      })
      .collect()
  }
}
