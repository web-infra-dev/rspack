use std::{
  cell::UnsafeCell,
  future::Future,
  pin::Pin,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
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

  let rx = Receiver {
    inner: inner.clone(),
  };

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

    unsafe {
      *self.inner.value.get() = Some(t);
    }

    self.inner.sent.store(true, Ordering::Release);
    Ok(())
  }
}

pub struct Receiver<T> {
  inner: Arc<Inner<T>>,
}

impl<T> Future for Receiver<T> {
  type Output = Result<T, ()>;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    loop {
      if self.inner.sent.load(Ordering::Acquire) {
        let value = unsafe { (*self.inner.value.get()).take() };
        if let Some(value) = value {
          return Poll::Ready(Ok(value));
        } else {
          return Poll::Ready(Err(()));
        }
      }
    }
  }
}
