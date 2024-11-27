use std::{any::Any, ptr::NonNull};

use rkyv::{
  de::{ErasedPtr, Pooling, PoolingState},
  ser::{sharing::SharingState, Sharing},
};

use crate::{DeserializeError, SerializeError};

const CONTEXT_ADDR: usize = 0;
unsafe fn default_drop(_: ErasedPtr) {}

/// A context wrapper that provides shared context methods
pub struct ContextGuard<'a> {
  context: &'a dyn Any,
}

impl<'a> ContextGuard<'a> {
  pub fn new(context: &'a dyn Any) -> Self {
    Self { context }
  }

  pub fn add_to_sharing<S: Sharing<SerializeError> + ?Sized>(
    &self,
    sharing: &mut S,
  ) -> Result<(), SerializeError> {
    sharing.start_sharing(CONTEXT_ADDR);
    sharing.finish_sharing(CONTEXT_ADDR, self as *const _ as usize)
  }

  pub fn sharing_context<S: Sharing<SerializeError> + ?Sized>(
    sharing: &'a mut S,
  ) -> Result<&'a dyn Any, SerializeError> {
    match sharing.start_sharing(CONTEXT_ADDR) {
      SharingState::Finished(addr) => {
        let guard: &Self = unsafe { &*(addr as *const Self) };
        Ok(guard.context)
      }
      _ => Err(SerializeError::NoContext),
    }
  }

  pub fn add_to_pooling<P: Pooling<DeserializeError> + ?Sized>(
    &self,
    pooling: &mut P,
  ) -> Result<(), DeserializeError> {
    unsafe {
      let ctx_ptr = ErasedPtr::new(NonNull::new_unchecked(self as *const _ as *mut ()));
      pooling.start_pooling(CONTEXT_ADDR);
      pooling.finish_pooling(CONTEXT_ADDR, ctx_ptr, default_drop)
    }
  }

  pub fn pooling_context<P: Pooling<DeserializeError> + ?Sized>(
    pooling: &'a mut P,
  ) -> Result<&'a dyn Any, DeserializeError> {
    match pooling.start_pooling(CONTEXT_ADDR) {
      PoolingState::Finished(ptr) => {
        let guard: &Self = unsafe { &*(ptr.data_address() as *const Self) };
        Ok(guard.context)
      }
      _ => Err(DeserializeError::NoContext),
    }
  }
}
