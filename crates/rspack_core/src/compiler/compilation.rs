use std::{
  cell::Cell,
  fmt,
  ops::{Deref, DerefMut},
  sync::{Arc, UniqueArc, Weak},
};

use either::Either;
use rspack_error::{Result, error};

use crate::Compilation;

type Ownership = Either<Arc<Compilation>, UniqueArc<Compilation>>;

/// Compiler-local ownership slot. Hooks and passes borrow `Compilation` as usual.
/// Only scheduling boundaries publish shared readers and recover unique ownership.
/// Workers receive an Arc rather than the ownership slot.
pub struct CompilationCell {
  // None is only used during a synchronous ownership conversion, never across
  // an await or a call into plugin code.
  owner: Cell<Option<Ownership>>,
}

// SAFETY: the Cell is private and every mutation requires &mut CompilationCell.
// Shared access only reads its initialized owner and the Sync Compilation. Never
// add a transition through &self: that would invalidate this implementation and
// the references returned by ownership()/Deref.
unsafe impl Sync for CompilationCell {}

impl CompilationCell {
  pub fn new(compilation: Compilation) -> Self {
    Self {
      owner: Cell::new(Some(Either::Right(UniqueArc::new(compilation)))),
    }
  }

  fn ownership(&self) -> &Ownership {
    // SAFETY: owner is private and every operation that changes it requires
    // &mut self. No shared method mutates the Cell, so this reference cannot
    // overlap a transition. The allocation is retained in either variant.
    unsafe { &*self.owner.as_ptr() }
      .as_ref()
      .expect("compilation ownership conversion must be synchronous")
  }

  /// Publishes an owned reader at the start of a shared scheduling phase.
  pub fn share(&mut self) -> Arc<Compilation> {
    let owner = self.owner.get_mut();
    let compilation = match owner.take().expect("compilation owner must exist") {
      Either::Left(compilation) => compilation,
      Either::Right(compilation) => UniqueArc::into_arc(compilation),
    };
    *owner = Some(Either::Left(compilation));
    let Some(Either::Left(compilation)) = owner else {
      unreachable!();
    };
    Arc::clone(compilation)
  }

  /// Recovers exclusive ownership after all reader tasks have finished.
  /// On failure the shared owner stays in the slot, so the operation can be retried.
  pub fn try_make_unique(&mut self) -> Result<()> {
    let owner = self.owner.get_mut();
    match owner.take().expect("compilation owner must exist") {
      Either::Right(compilation) => {
        *owner = Some(Either::Right(compilation));
        Ok(())
      }
      Either::Left(compilation) => {
        // SAFETY: Compilation is a concrete type with no lifetime parameters;
        // all weak pointers to the allocation have the same inner type.
        match unsafe { rspack_util::arc::try_into_unique(compilation) } {
          Ok(compilation) => {
            *owner = Some(Either::Right(compilation));
            Ok(())
          }
          Err(compilation) => {
            *owner = Some(Either::Left(compilation));
            Err(error!(
              "Cannot modify compilation while Arc readers are still alive"
            ))
          }
        }
      }
    }
  }

  pub fn downgrade(&self) -> Weak<Compilation> {
    match self.ownership() {
      Either::Left(compilation) => Arc::downgrade(compilation),
      Either::Right(compilation) => UniqueArc::downgrade(compilation),
    }
  }
}

impl Deref for CompilationCell {
  type Target = Compilation;

  fn deref(&self) -> &Compilation {
    match self.ownership() {
      Either::Left(compilation) => compilation,
      Either::Right(compilation) => compilation,
    }
  }
}

impl DerefMut for CompilationCell {
  fn deref_mut(&mut self) -> &mut Compilation {
    match self.owner.get_mut() {
      Some(Either::Right(compilation)) => compilation,
      _ => panic!("scheduler must recover UniqueArc before mutating compilation"),
    }
  }
}

impl fmt::Debug for CompilationCell {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("CompilationCell")
      .field(self.ownership())
      .finish()
  }
}
