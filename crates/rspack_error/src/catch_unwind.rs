use std::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt};

use super::{internal_error, Result};

pub fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R> {
  match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
    Ok(res) => Ok(res),
    Err(cause) => match cause.downcast_ref::<&'static str>() {
      None => match cause.downcast_ref::<String>() {
        None => Err(internal_error!("Unknown panic message")),
        Some(message) => Err(internal_error!("{message}")),
      },
      Some(message) => Err(internal_error!("{message}")),
    },
  }
}

pub struct CatchUnwindFuture<F: Future + Send + 'static> {
  inner: BoxFuture<'static, F::Output>,
}

impl<F: Future + Send + 'static> CatchUnwindFuture<F> {
  pub fn create(f: F) -> Self {
    Self { inner: f.boxed() }
  }
}

impl<F: Future + Send + 'static> Future for CatchUnwindFuture<F> {
  type Output = Result<F::Output>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let inner = &mut self.inner;

    match catch_unwind(move || inner.poll_unpin(cx)) {
      Ok(Poll::Pending) => Poll::Pending,
      Ok(Poll::Ready(value)) => Poll::Ready(Ok(value)),
      Err(cause) => Poll::Ready(Err(cause)),
    }
  }
}
