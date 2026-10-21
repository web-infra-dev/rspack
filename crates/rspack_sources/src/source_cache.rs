use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::BoxSource;

/// Release CachedSource memoization across source roots, visiting shared nodes once.
/// Content and maps remain readable; caches are recreated on demand. Maps whose
/// fields have been borrowed by streaming callbacks retain their stable storage.
#[derive(Default)]
pub struct SourceCacheClearer {
  sources: FxHashSet<usize>,
  pub(crate) maps: FxHashSet<usize>,
}

impl SourceCacheClearer {
  /// Clear a source and its reachable children without materializing their content.
  pub fn clear(&mut self, source: &BoxSource) {
    let address = Arc::as_ptr(source) as *const () as usize;
    if self.sources.insert(address) {
      source.as_ref().clear_cached_sources(self);
    }
  }
}
