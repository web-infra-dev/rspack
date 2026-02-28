//! Busy-wait version oneshot channel aiming to be the drop-in replacement of other oneshot crates such [tokio::sync::oneshot]
//!
//! When [Receiver] start to receive before [Sender::send], normally the receiver will be put into the task pool and be waken by the [Sender].
//! However, runtime should acquire the lock of task pool before waking the receiver, which is forbidden in the main thread of browser.
//! So the busy-wait [Receiver] is provided.
use std::{
  cell::UnsafeCell,
  future::Future,
  pin::Pin,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  task::{Context, Poll},
};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
  let inner = Arc::new(Inner {
    sent: AtomicBool::new(false),
    value: UnsafeCell::new(None),
  });

  let tx = Sender {
    inner: inner.clone(),
  };

  let rx = Receiver { inner };

  (tx, rx)
}

struct Inner<T> {
  sent: AtomicBool,
  value: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Send> Sync for Inner<T> {}

pub struct Sender<T> {
  inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
  pub fn send(self, t: T) -> Result<(), T> {
    if self.inner.sent.load(Ordering::Acquire) {
      return Err(t);
    }

    // SAFETY: `send` can only be called once.
    unsafe {
      *self.inner.value.get() = Some(t);
    }

    self.inner.sent.store(true, Ordering::Release);
    Ok(())
  }
}

impl<T> Drop for Sender<T> {
  fn drop(&mut self) {
    if !self.inner.sent.load(Ordering::Acquire) {
      self.inner.sent.store(true, Ordering::Relaxed);
    }
  }
}

pub struct Receiver<T> {
  inner: Arc<Inner<T>>,
}

impl<T> Future for Receiver<T> {
  type Output = Result<T, ()>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if self.inner.sent.load(Ordering::Acquire) {
      let value = unsafe { (*self.inner.value.get()).take() };
      if let Some(value) = value {
        Poll::Ready(Ok(value))
      } else {
        Poll::Ready(Err(()))
      }
    } else {
      // Give tokio the chance to run other tasks
      cx.waker().wake_by_ref();
      Poll::Pending
    }
  }
}
