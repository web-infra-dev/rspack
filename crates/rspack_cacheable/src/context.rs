use std::{any::Any, ops::Deref, path::Path, ptr::NonNull};

use rkyv::{
  de::{ErasedPtr, Pooling, PoolingState},
  ser::{Sharing, sharing::SharingState},
};

use crate::error::{Error, Result};

const CONTEXT_ADDR: usize = 0;
unsafe fn default_drop(_: ErasedPtr) {}

pub trait CacheableContext: Any {
  fn project_root(&self) -> Option<&Path>;
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
}

impl<'a> Deref for ContextGuard<'a> {
  type Target = dyn CacheableContext;
  fn deref(&self) -> &Self::Target {
    self.context
  }
}

impl<'a> ContextGuard<'a> {
  pub fn new(context: &'a dyn CacheableContext) -> Self {
    Self { context }
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
      let ctx_ptr = ErasedPtr::new(NonNull::new_unchecked(self as *const _ as *mut ()));
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
