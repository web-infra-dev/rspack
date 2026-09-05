use std::sync::Mutex;

use rspack_collections::IdentifierMap;

use crate::ModuleIdentifier;

/// Work belonging to one graph's Make invocation, shared by its successive updates.
/// Each compilation creates separate sessions for its main and execution graphs,
/// while both graphs use the same module build cache.
#[derive(Debug, Default)]
pub(crate) struct MakeSession {
  pending_cache_writes: Mutex<IdentifierMap<u64>>,
}

impl MakeSession {
  pub(crate) fn record_build(&self, identifier: ModuleIdentifier, started_at: u64) {
    self
      .pending_cache_writes
      .lock()
      .expect("should lock Make cache writes")
      .insert(identifier, started_at);
  }

  pub(crate) fn take_cache_writes(&self) -> IdentifierMap<u64> {
    std::mem::take(
      &mut *self
        .pending_cache_writes
        .lock()
        .expect("should lock Make cache writes"),
    )
  }
}
