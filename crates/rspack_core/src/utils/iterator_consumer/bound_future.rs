use std::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

/// A future bound to lifetime `'a` that panics if dropped before completion,
/// guaranteeing every spawned task finishes before `'a`-borrowed data is freed.
pub struct BoundFuture<'a, T> {
  inner: Pin<Box<dyn Future<Output = T> + Send + 'a>>,
  /// Set to `true` once `poll` returns `Ready`, used by `Drop` to detect early drops.
  completed: bool,
}

impl<'a, T> BoundFuture<'a, T> {
  /// Wraps `fut` and binds it to lifetime `'a`.
  pub fn new<F>(fut: F) -> Self
  where
    F: Future<Output = T> + Send + 'a,
  {
    Self {
      inner: Box::pin(fut),
      completed: false,
    }
  }
}

impl<T> Future for BoundFuture<'_, T> {
  type Output = T;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
    let result = self.inner.as_mut().poll(cx);
    if result.is_ready() {
      self.completed = true;
    }
    result
  }
}

impl<T> Drop for BoundFuture<'_, T> {
  fn drop(&mut self) {
    if !self.completed {
      panic!(
        "BoundFuture dropped without being awaited to completion; \
         this would cause use-after-free in the spawned tasks"
      );
    }
  }
}
