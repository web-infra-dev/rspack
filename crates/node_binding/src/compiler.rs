use std::{
  marker::PhantomPinned,
  ops::{Deref, DerefMut},
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
