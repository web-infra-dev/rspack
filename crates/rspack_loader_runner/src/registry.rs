use std::sync::{Arc, LazyLock, RwLock, Weak};

use crate::LoaderContextId;

type DropListener = dyn Fn(LoaderContextId) + Send + Sync;
static LISTENERS: LazyLock<RwLock<Vec<Weak<DropListener>>>> = LazyLock::new(Default::default);

/// Keep the returned callback alive for as long as notifications are needed.
pub fn register_loader_context_drop_listener(
  listener: impl Fn(LoaderContextId) + Send + Sync + 'static,
) -> Arc<DropListener> {
  let listener: Arc<DropListener> = Arc::new(listener);
  let mut listeners = LISTENERS.write().expect("loader context registry lock");
  listeners.retain(|listener| listener.strong_count() > 0);
  listeners.push(Arc::downgrade(&listener));
  listener
}

pub(crate) fn notify_loader_context_drop(id: LoaderContextId) {
  for listener in LISTENERS
    .read()
    .expect("loader context registry lock")
    .iter()
  {
    if let Some(listener) = listener.upgrade() {
      listener(id);
    }
  }
}
