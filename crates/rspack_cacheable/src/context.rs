use std::{any::Any, cell::Cell, ops::Deref, path::Path};

use rkyv::{
  de::{ErasedPtr, Pooling, PoolingState},
  ser::{Sharing, sharing::SharingState},
};

use crate::{
  cacheable,
  error::{Error, Result},
};

const CONTEXT_ADDR: usize = 0;
unsafe fn default_drop(_: ErasedPtr) {}

pub trait CacheableContext: Any {
  /// Returns the project root used for portable path conversion, spelled as `context` spells it.
  ///
  /// `Some` enables portable conversion relative to this root, while `None` keeps paths in their
  /// native representation.
  fn project_root(&self) -> Option<&Path>;

  /// Returns the realpath of [`Self::project_root`]. The resolver realpaths resources, so paths
  /// under this spelling are rebased as well and restore in this spelling.
  fn canonical_project_root(&self) -> Option<&Path> {
    self.project_root()
  }
}

/// A spelling of the project root that a portable path is stored relative to.
#[cacheable(crate=crate, hashable)]
#[derive(Debug, Clone, Copy)]
pub enum RootSpelling {
  /// The project root as `context` spells it.
  Context,
  /// The realpath of the project root.
  Realpath,
}

// Implement for unit type for convenience in tests and simple cases
impl CacheableContext for () {
  fn project_root(&self) -> Option<&Path> {
    None
  }
}

/// A context wrapper that provides shared context methods
pub struct ContextGuard<'a> {
  context: &'a dyn CacheableContext,
  saw_outside_root: Cell<bool>,
}

impl<'a> Deref for ContextGuard<'a> {
  type Target = dyn CacheableContext;
  fn deref(&self) -> &Self::Target {
    self.context
  }
}

impl<'a> ContextGuard<'a> {
  pub fn new(context: &'a dyn CacheableContext) -> Self {
    Self {
      context,
      saw_outside_root: Cell::new(false),
    }
  }

  pub fn mark_outside_root(&self) {
    self.saw_outside_root.set(true);
  }

  pub fn saw_outside_root(&self) -> bool {
    self.saw_outside_root.get()
  }

  /// Spellings of the project root to rebase against, longest first so that a spelling nested
  /// in the other wins. Equal spellings are reported once, as [`RootSpelling::Realpath`], because
  /// resolving a resource again produces the realpath.
  pub fn root_spellings(&self) -> impl Iterator<Item = (RootSpelling, &Path)> {
    let realpath = self.canonical_project_root();
    let context = self.project_root().filter(|root| Some(*root) != realpath);
    let context_first = context
      .zip(realpath)
      .is_some_and(|(context, realpath)| context.as_os_str().len() > realpath.as_os_str().len());
    let realpath = realpath.map(|root| (RootSpelling::Realpath, root));
    let context = context.map(|root| (RootSpelling::Context, root));
    let (first, second) = if context_first {
      (context, realpath)
    } else {
      (realpath, context)
    };
    first.into_iter().chain(second)
  }

  /// Returns the project root in `spelling`, or `None` when portable conversion is disabled.
  pub fn root(&self, spelling: RootSpelling) -> Option<&Path> {
    match spelling {
      RootSpelling::Context => self.project_root(),
      RootSpelling::Realpath => self.canonical_project_root(),
    }
  }

  pub fn add_to_sharing<S: Sharing<Error> + ?Sized>(&self, sharing: &mut S) -> Result<()> {
    sharing.start_sharing(CONTEXT_ADDR);
    sharing.finish_sharing(CONTEXT_ADDR, self as *const _ as usize)
  }

  pub fn sharing_guard<S: Sharing<Error> + ?Sized>(sharing: &'a mut S) -> Result<&'a Self> {
    match sharing.start_sharing(CONTEXT_ADDR) {
      SharingState::Finished(addr) => {
        let guard: &Self = unsafe { &*(addr as *const Self) };
        Ok(guard)
      }
      _ => Err(Error::NoContext),
    }
  }

  pub fn add_to_pooling<P: Pooling<Error> + ?Sized>(&self, pooling: &mut P) -> Result<()> {
    unsafe {
      let ctx_ptr = ErasedPtr::new(self as *const _ as *mut ());
      pooling.start_pooling(CONTEXT_ADDR);
      pooling.finish_pooling(CONTEXT_ADDR, ctx_ptr, default_drop)
    }
  }

  pub fn pooling_guard<P: Pooling<Error> + ?Sized>(pooling: &'a mut P) -> Result<&'a Self> {
    match pooling.start_pooling(CONTEXT_ADDR) {
      PoolingState::Finished(ptr) => {
        let guard: &Self = unsafe { &*(ptr.data_address() as *const Self) };
        Ok(guard)
      }
      _ => Err(Error::NoContext),
    }
  }

  pub fn downcast_context<T: 'static>(&'a self) -> Result<&'a T> {
    (self.context as &dyn Any)
      .downcast_ref()
      .ok_or(Error::NoContext)
  }
}
