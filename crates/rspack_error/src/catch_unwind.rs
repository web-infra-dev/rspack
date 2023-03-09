use std::{
  backtrace::Backtrace,
  cell::RefCell,
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt};

use super::{internal_error, Result};

#[allow(non_snake_case)]
pub mod PanicStrategy {
  /// Strategy for panic handling.
  /// [`PanicStrategy::Suppressed`] means that panic for `catch_unwind` is suppressed.
  /// [`PanicStrategy::NotSuppressed`] means that panic for `catch_unwind` is not suppressed.
  pub trait S: 'static + Unpin + Send + Sync {
    fn is_suppressed() -> bool;
  }

  /// Panic for `catch_unwind` is suppressed. But it is not
  /// suppressed for those which are not wrapped by `catch_unwind`.
  pub struct Suppressed;

  impl S for Suppressed {
    #[inline]
    fn is_suppressed() -> bool {
      true
    }
  }

  /// Panic for `catch_unwind` is not suppressed.
  /// Every panic will be preserved and propagated to the panic hook set before.
  pub struct NotSuppressed;

  impl S for NotSuppressed {
    #[inline]
    fn is_suppressed() -> bool {
      false
    }
  }
}

#[inline]
fn panic_hook_handler<S: PanicStrategy::S, R>(f: impl FnOnce() -> R) -> R {
  PANIC_HOOK.with(|hook| {
    if !S::is_suppressed() {
      *hook.borrow_mut() = Some(std::panic::take_hook());
    }
  });
  std::panic::set_hook(Box::new(move |info| {
    PANIC_HOOK.with(|hook| {
      if let Some(hook) = &*hook.borrow() {
        hook(info);
      }
    });
    BACKTRACE.with(|bt| {
      *bt.borrow_mut() = Some(std::backtrace::Backtrace::force_capture());
    });
  }));
  let result = f();
  PANIC_HOOK.with(|hook| {
    if let Some(hook) = hook.borrow_mut().take() {
      std::panic::set_hook(hook);
    }
  });

  result
}

thread_local! {
  static BACKTRACE: RefCell<Option<Backtrace>> = RefCell::new(None);
  static PANIC_HOOK: RefCell<Option<Box<dyn Fn(&std::panic::PanicInfo<'_>) + 'static + Sync + Send>>> = RefCell::new(None);
}

pub fn catch_unwind<S: PanicStrategy::S, R>(f: impl FnOnce() -> R) -> Result<R> {
  match panic_hook_handler::<S, _>(move || {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
  }) {
    Ok(res) => Ok(res),
    Err(cause) => {
      let backtrace = BACKTRACE
        .with(|b| b.borrow().as_ref().map(|b| b.to_string()))
        .unwrap_or_default();

      match cause.downcast_ref::<&'static str>() {
        None => match cause.downcast_ref::<String>() {
          None => Err(internal_error!(
            "Unknown fatal error.\n {GENERIC_FATAL_MESSAGE}\n\n{backtrace}"
          )),
          Some(message) => Err(internal_error!(
            "Fatal error encountered: {message}.\n{GENERIC_FATAL_MESSAGE}\n\n{backtrace}"
          )),
        },
        Some(message) => Err(internal_error!(
          "Fatal error encountered: {message}.\n{GENERIC_FATAL_MESSAGE}\n\n{backtrace}"
        )),
      }
    }
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

    match catch_unwind::<PanicStrategy::Suppressed, _>(move || inner.poll_unpin(cx)) {
      Ok(Poll::Pending) => Poll::Pending,
      Ok(Poll::Ready(value)) => Poll::Ready(Ok(value)),
      Err(cause) => Poll::Ready(Err(cause)),
    }
  }
}

const GENERIC_FATAL_MESSAGE: &str =
  "This is not expected, please file an issue at https://github.com/web-infra-dev/rspack/issues.";
