use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use napi::{
  Status,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use rustc_hash::FxHashMap;

static RELEASE: OnceLock<Arc<ThreadsafeFunction<u32, (), u32, Status, false, true, 0>>> =
  OnceLock::new();

struct MainObjectHandleInner {
  handle: u32,
  release: Arc<ThreadsafeFunction<u32, (), u32, Status, false, true, 0>>,
}

impl Drop for MainObjectHandleInner {
  fn drop(&mut self) {
    self
      .release
      .call(self.handle, ThreadsafeFunctionCallMode::NonBlocking);
  }
}

/// A cloneable handle for a JavaScript value owned by the main thread.
///
/// The last clone releases the handle through a threadsafe main-thread callback.
#[derive(Clone)]
pub struct MainObjectHandle {
  inner: Arc<MainObjectHandleInner>,
}

impl MainObjectHandle {
  pub fn register_release(release: ThreadsafeFunction<u32, (), u32, Status, false, true, 0>) {
    let _ = RELEASE.set(Arc::new(release));
  }

  pub fn new(handle: u32) -> Self {
    Self {
      inner: Arc::new(MainObjectHandleInner {
        handle,
        release: RELEASE
          .get()
          .expect("main-thread JS value release callback should be registered")
          .clone(),
      }),
    }
  }

  pub fn handle(&self) -> u32 {
    self.inner.handle
  }
}

static REFERENCES: LazyLock<Mutex<FxHashMap<u32, FxHashMap<String, bool>>>> =
  LazyLock::new(Default::default);

#[napi]
pub fn register_main_object_release(
  release: ThreadsafeFunction<u32, (), u32, Status, false, true, 0>,
) {
  MainObjectHandle::register_release(release);
}

#[napi]
pub fn register_loader_reference(owner: u32, ident: String, parallel: bool) {
  REFERENCES
    .lock()
    .expect("loader references lock poisoned")
    .entry(owner)
    .or_default()
    .insert(ident, parallel);
}

#[napi]
pub fn release_loader_references(owner: u32) {
  REFERENCES
    .lock()
    .expect("loader references lock poisoned")
    .remove(&owner);
}

pub(super) fn is_parallel(owner: u32, query: Option<&str>) -> bool {
  let Some(ident) = query.and_then(|q| q.strip_prefix("??")) else {
    return false;
  };
  REFERENCES
    .lock()
    .expect("loader references lock poisoned")
    .get(&owner)
    .and_then(|references| references.get(ident))
    .copied()
    .unwrap_or(false)
}
