use std::{
  future::Future,
  marker::PhantomPinned,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicBool, Ordering},
};

use tokio::sync::watch;

type CompilerInner = rspack_core::Compiler;

/// `Compiler` struct that is `!Unpin`.
pub(crate) struct Compiler(CompilerInner, PhantomPinned);

impl From<CompilerInner> for Compiler {
  fn from(value: CompilerInner) -> Self {
    Self(value, PhantomPinned)
  }
}

impl Deref for Compiler {
  type Target = CompilerInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Compiler {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub(crate) struct CompilerState {
  running: watch::Sender<bool>,
  closing: AtomicBool,
}

impl CompilerState {
  pub(crate) fn init() -> Self {
    Self {
      running: watch::Sender::new(false),
      closing: AtomicBool::new(false),
    }
  }
}

impl CompilerState {
  pub(crate) fn running(&self) -> bool {
    *self.running.borrow()
  }

  pub(crate) fn closing(&self) -> bool {
    self.closing.load(Ordering::Acquire)
  }

  pub(crate) fn begin_close(&self) -> bool {
    !self.closing.swap(true, Ordering::AcqRel)
  }

  pub(crate) fn enter(&self) -> CompilerStateGuard {
    self.running.send_replace(true);
    CompilerStateGuard(self.running.clone())
  }

  /// Resolves once no build/rebuild is in flight. Detached from `self` so callers
  /// can await it without borrowing the compiler.
  pub(crate) fn wait_idle(&self) -> impl Future<Output = ()> + Send + 'static {
    let mut receiver = self.running.subscribe();
    async move {
      // `wait_for` checks the current value first, so an idle compiler resolves
      // immediately. A closed channel means the compiler is dropped, hence idle.
      let _ = receiver.wait_for(|running| !running).await;
    }
  }
}

pub(crate) struct CompilerStateGuard(watch::Sender<bool>);

impl Drop for CompilerStateGuard {
  fn drop(&mut self) {
    self.0.send_replace(false);
  }
}
