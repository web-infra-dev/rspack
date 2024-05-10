use std::{
  marker::PhantomPinned,
  ops::{Deref, DerefMut},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use rspack_fs_node::AsyncNodeWritableFileSystem;

type CompilerInner = rspack_core::Compiler<AsyncNodeWritableFileSystem>;

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

pub(crate) struct CompilerState(Arc<AtomicBool>);

impl CompilerState {
  pub(crate) fn init() -> Self {
    Self(Arc::new(AtomicBool::new(false)))
  }
}

impl CompilerState {
  pub(crate) fn running(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }

  pub(crate) fn enter(&self) -> CompilerStateGuard {
    self.0.store(true, Ordering::Relaxed);
    CompilerStateGuard(self.0.clone())
  }
}

pub(crate) struct CompilerStateGuard(Arc<AtomicBool>);

unsafe impl Send for CompilerStateGuard {}

impl Drop for CompilerStateGuard {
  fn drop(&mut self) {
    self.0.store(false, Ordering::Relaxed);
  }
}
