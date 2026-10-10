use std::sync::{
  Arc, LazyLock, RwLock,
  atomic::{AtomicU32, AtomicU64, Ordering},
};

use rustc_hash::FxHashMap;

/// Process-wide identity of a single loader run, including all its JS yields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoaderContextId(pub u32);

static NEXT_CONTEXT_ID: AtomicU32 = AtomicU32::new(0);
static NEXT_LISTENER_ID: AtomicU64 = AtomicU64::new(0);
type DropListener = Arc<dyn Fn(LoaderContextId) + Send + Sync>;
static LISTENERS: LazyLock<RwLock<FxHashMap<u64, DropListener>>> = LazyLock::new(Default::default);

/// Registers a listener for loader context destruction. Dropping the registration
/// unregisters it, so environments cannot retain callbacks after shutdown.
pub fn register_loader_context_drop_listener(
  listener: impl Fn(LoaderContextId) + Send + Sync + 'static,
) -> LoaderContextDropRegistration {
  let id = NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed);
  LISTENERS
    .write()
    .expect("loader context registry lock")
    .insert(id, Arc::new(listener));
  LoaderContextDropRegistration(id)
}

pub struct LoaderContextDropRegistration(u64);
impl Drop for LoaderContextDropRegistration {
  fn drop(&mut self) {
    LISTENERS
      .write()
      .expect("loader context registry lock")
      .remove(&self.0);
  }
}

// A field guard allows LoaderResult to move the context's fields while still
// notifying listeners when the loader run ends (or is cancelled).
#[derive(Debug)]
pub(crate) struct LoaderContextLifetime(LoaderContextId);
impl LoaderContextLifetime {
  pub(crate) fn new() -> Self {
    let id = NEXT_CONTEXT_ID
      .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
      .expect("loader context id exhausted");
    Self(LoaderContextId(id))
  }
  pub(crate) fn id(&self) -> LoaderContextId {
    self.0
  }
}
/// Notifies foreign runners that a context can no longer be retained. This also
/// allows an abandoned ownership transfer to release its foreign references.
pub fn notify_loader_context_drop(id: LoaderContextId) {
  let listeners: Vec<_> = LISTENERS
    .read()
    .expect("loader context registry lock")
    .values()
    .cloned()
    .collect();
  for listener in listeners {
    listener(id);
  }
}

impl Drop for LoaderContextLifetime {
  fn drop(&mut self) {
    notify_loader_context_drop(self.0);
  }
}
