use std::{
  cell::UnsafeCell,
  future::Future,
  pin::Pin,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  task::{Context, Poll},
  time::Duration,
};

use tokio::time::Sleep;

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
    state: RecvState::Checking,
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
  state: RecvState,
}

enum RecvState {
  Checking,
  Sleeping(Pin<Box<Sleep>>),
}

impl<T> Future for Receiver<T> {
  type Output = Result<T, ()>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    loop {
      match &mut self.state {
        RecvState::Checking => {
          if self.inner.sent.load(Ordering::Acquire) {
            let value = unsafe { (*self.inner.value.get()).take() };
            if let Some(value) = value {
              return Poll::Ready(Ok(value));
            } else {
              return Poll::Ready(Err(()));
            }
          }
          self.state =
            RecvState::Sleeping(Box::pin(tokio::time::sleep(Duration::from_millis(100))));
        }
        RecvState::Sleeping(sleep_future) => match sleep_future.as_mut().poll(cx) {
          Poll::Ready(()) => {
            self.state = RecvState::Checking;
            continue;
          }
          Poll::Pending => return Poll::Pending,
        },
      }
    }
  }
}
