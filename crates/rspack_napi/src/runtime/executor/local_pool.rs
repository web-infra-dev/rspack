use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::thread::Thread;
use std::vec::Vec;

use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use futures::task::waker_ref;
use futures::task::ArcWake;
use futures::task::Context;
use futures::task::FutureObj;
use futures::task::LocalFutureObj;
use futures::task::LocalSpawn;
use futures::task::Poll;
use futures::task::Spawn;
use futures::task::SpawnError;

use super::enter::enter;

#[derive(Debug)]
pub struct LocalPool {
  pool: FuturesUnordered<LocalFutureObj<'static, ()>>,
  incoming: Rc<Incoming>,
}

#[derive(Clone, Debug)]
pub struct LocalSpawner {
  incoming: Weak<Incoming>,
}

type Incoming = RefCell<Vec<LocalFutureObj<'static, ()>>>;

#[derive(Debug)]
pub(crate) struct ThreadNotify {
  pub(crate) thread: Thread,
  pub(crate) unparked: AtomicBool,
}

pub type ThreadNotifyRef = Arc<ThreadNotify>;

impl ThreadNotify {
  pub fn new() -> Arc<Self> {
    Arc::new(ThreadNotify {
      thread: thread::current(),
      unparked: AtomicBool::new(false),
    })
  }
}

impl ArcWake for ThreadNotify {
  fn wake_by_ref(arc_self: &Arc<Self>) {
    let unparked = arc_self.unparked.swap(true, Ordering::Release);
    if !unparked {
      arc_self.thread.unpark();
    }
  }
}

pub fn wait_for_wake(thread_notify: &ThreadNotify) {
  while !thread_notify.unparked.swap(false, Ordering::Acquire) {
    std::thread::park();
  }
}

fn woken(thread_notify: &ThreadNotify) -> bool {
  thread_notify.unparked.load(Ordering::Acquire)
}

fn run_executor<T, F: FnMut(&mut Context<'_>) -> Poll<T>>(
  thread_notify: Arc<ThreadNotify>,
  mut f: F,
) -> T {
  let _enter = enter().expect(
    "cannot execute `LocalPool` executor from within \
         another executor",
  );

  let waker = waker_ref(&thread_notify);
  let mut cx = Context::from_waker(&waker);

  loop {
    if let Poll::Ready(t) = f(&mut cx) {
      return t;
    }

    // Wait for a wakeup.
    while !thread_notify.unparked.swap(false, Ordering::Acquire) {
      // No wakeup occurred. It may occur now, right before parking,
      // but in that case the token made available by `unpark()`
      // is guaranteed to still be available and `park()` is a no-op.
      thread::park();
    }
  }
}

impl LocalPool {
  pub fn new() -> Self {
    Self {
      pool: FuturesUnordered::new(),
      incoming: Default::default(),
    }
  }

  pub fn spawner(&self) -> LocalSpawner {
    LocalSpawner {
      incoming: Rc::downgrade(&self.incoming),
    }
  }

  // Note: Can be used to interleave futures with the JS event loop
  /// Runs all tasks and returns after completing one future or until no more progress
  /// can be made. Returns `true` if one future was completed, `false` otherwise.
  #[allow(unused)]
  pub fn try_run_one(&mut self, thread_notify: Arc<ThreadNotify>) -> bool {
    run_executor(thread_notify.clone(), |cx| {
      loop {
        self.drain_incoming();

        match self.pool.poll_next_unpin(cx) {
          // Success!
          Poll::Ready(Some(())) => return Poll::Ready(true),
          // The pool was empty.
          Poll::Ready(None) => return Poll::Ready(false),
          Poll::Pending => (),
        }

        if !self.incoming.borrow().is_empty() {
          // New tasks were spawned; try again.
          continue;
        } else if woken(&thread_notify) {
          // The pool yielded to us, but there's more progress to be made.
          return Poll::Pending;
        } else {
          return Poll::Ready(false);
        }
      }
    })
  }

  /// Runs all tasks in the pool and returns if no more progress can be made on any task.
  #[allow(unused)]
  pub fn run_until_stalled(&mut self, thread_notify: Arc<ThreadNotify>) {
    run_executor(thread_notify.clone(), |cx| match self.poll_pool(cx) {
      // The pool is empty.
      Poll::Ready(()) => Poll::Ready(()),
      Poll::Pending => {
        if woken(&thread_notify) {
          Poll::Pending
        } else {
          // We're stalled for now.
          Poll::Ready(())
        }
      }
    });
  }

  fn poll_pool(&mut self, cx: &mut Context<'_>) -> Poll<()> {
    loop {
      self.drain_incoming();
      let pool_ret = self.pool.poll_next_unpin(cx);

      // We queued up some new tasks; add them and poll again.
      if !self.incoming.borrow().is_empty() {
        continue;
      }

      match pool_ret {
        Poll::Ready(Some(())) => continue,
        Poll::Ready(None) => return Poll::Ready(()),
        Poll::Pending => return Poll::Pending,
      }
    }
  }

  fn drain_incoming(&mut self) {
    let mut incoming = self.incoming.borrow_mut();
    for task in incoming.drain(..) {
      self.pool.push(task)
    }
  }
}

impl Default for LocalPool {
  fn default() -> Self {
    Self::new()
  }
}

impl Spawn for LocalSpawner {
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
    if let Some(incoming) = self.incoming.upgrade() {
      incoming.borrow_mut().push(future.into());
      Ok(())
    } else {
      Err(SpawnError::shutdown())
    }
  }

  fn status(&self) -> Result<(), SpawnError> {
    if self.incoming.upgrade().is_some() {
      Ok(())
    } else {
      Err(SpawnError::shutdown())
    }
  }
}

impl LocalSpawn for LocalSpawner {
  fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
    if let Some(incoming) = self.incoming.upgrade() {
      incoming.borrow_mut().push(future);
      Ok(())
    } else {
      Err(SpawnError::shutdown())
    }
  }

  fn status_local(&self) -> Result<(), SpawnError> {
    if self.incoming.upgrade().is_some() {
      Ok(())
    } else {
      Err(SpawnError::shutdown())
    }
  }
}
