use std::{
  fmt::Write,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use rspack_util::fx_hash::FxDashMap;

use crate::Logger;

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

  pub fn report(&self, compiler_path: &str, full_hash: Option<&str>, logger: &impl Logger) {
    // All compilation work has joined before taking this snapshot.
    let mut entries = self
      .entries
      .iter()
      .filter_map(|entry| {
        let counters = entry.value();
        let hits = counters.hits.load(Ordering::Relaxed);
        let misses = counters.misses.load(Ordering::Relaxed);
        let reused = counters.reused.load(Ordering::Relaxed);
        let stores = counters.stores.load(Ordering::Relaxed);
        (hits != 0 || misses != 0 || reused != 0 || stores != 0)
          .then(|| (Arc::clone(entry.key()), hits, misses, reused, stores))
      })
      .collect::<Vec<_>>();
    if entries.is_empty() {
      return;
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
    let mut raw = serde_json::Map::new();
    for (name, hits, misses, reused, stores) in entries {
      raw.insert(
        name.to_string(),
        serde_json::json!({ "hits": hits, "misses": misses, "reused": reused, "stores": stores }),
      );
      let name = name.strip_prefix(&prefix).unwrap_or(&name);
      let total = hits + misses;
      write!(message, "\n  {name}: ").expect("writing to a String cannot fail");
      if total != 0 {
        let rate = hits as f64 / total as f64 * 100.0;
        write!(message, "{rate:.1}% ({hits}/{total}) hits, ")
          .expect("writing to a String cannot fail");
      }
      write!(message, "{reused} reused, {stores} stores").expect("writing to a String cannot fail");
    }
    logger.log(message);
    logger.debug(serde_json::Value::Object(raw).to_string());
  }
}
