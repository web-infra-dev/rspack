use std::sync::{Arc, OnceLock};

use napi::{
  Status,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

static RELEASE: OnceLock<Arc<ThreadsafeFunction<u32, (), u32, Status, false, true, 0>>> =
  OnceLock::new();

struct MainThreadJsValueHandleInner {
  handle: u32,
  release: Arc<ThreadsafeFunction<u32, (), u32, Status, false, true, 0>>,
}

impl Drop for MainThreadJsValueHandleInner {
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
pub struct MainThreadJsValueHandle {
  inner: Arc<MainThreadJsValueHandleInner>,
}

impl MainThreadJsValueHandle {
  pub fn register_release(release: ThreadsafeFunction<u32, (), u32, Status, false, true, 0>) {
    let _ = RELEASE.set(Arc::new(release));
  }

  pub fn new(handle: u32) -> Self {
    Self {
      inner: Arc::new(MainThreadJsValueHandleInner {
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
