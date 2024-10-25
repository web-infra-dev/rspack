use std::sync::Arc;
// the block_on in future crate doesn't support recursively call, but we need recursively call now since
// we have nest async between sync call
pub fn block_on<F>(future: F) -> F::Output
where
  F: std::future::Future,
{
  use std::{
    task::{Context, Poll},
    thread::{self, Thread},
  };

  use futures_task::{waker_ref, ArcWake};
  struct ThreadWaker(Thread);

  impl ArcWake for ThreadWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
      arc_self.0.unpark();
    }
  }

  let waker = Arc::new(ThreadWaker(thread::current()));
  let waker = waker_ref(&waker);
  let mut cx = Context::from_waker(&waker);

  tokio::pin!(future);

  loop {
    match future.as_mut().poll(&mut cx) {
      Poll::Ready(ret) => return ret,
      Poll::Pending => thread::park(),
    }
  }
}
