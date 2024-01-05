use std::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt};

use super::{error, Result};

const GENERIC_FATAL_MESSAGE: &str =
  "This is not expected, please file an issue at https://github.com/web-infra-dev/rspack/issues.";

fn raise(message: &str) -> crate::Error {
  let backtrace = std::backtrace::Backtrace::force_capture().to_string();
  error!(
    r#"{message}
{GENERIC_FATAL_MESSAGE}
{backtrace}
"#
  )
}

pub fn catch_unwind<R>(f: impl FnOnce() -> R) -> Result<R> {
  match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
    Ok(res) => Ok(res),
    Err(cause) => match cause.downcast_ref::<&'static str>() {
      None => match cause.downcast_ref::<String>() {
        None => Err(raise("Unknown fatal error")),
        Some(message) => Err(raise(message)),
      },
      Some(message) => Err(raise(message)),
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
