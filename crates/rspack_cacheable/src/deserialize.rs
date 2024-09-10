use std::{any::Any, ptr::NonNull};

use rkyv::{
  access,
  api::{deserialize_using, high::HighValidator},
  bytecheck::CheckBytes,
  de::{ErasedPtr, Pool, Pooling},
  rancor::{BoxedError, Source, Strategy, Trace},
  Archive, Deserialize,
};

const CONTEXT_ADDR: usize = 2;
unsafe fn default_drop(_: ErasedPtr) {}

#[derive(Debug)]
pub enum DeserializeError {
  RkyvError(BoxedError),
  DynCheckBytesNotRegister,
  // A deserialize failed occurred
  DeserializeFailed(&'static str),
}

impl std::fmt::Display for DeserializeError {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //    write!(f, "{}", self.inner)?;
    Ok(())
  }
}

impl std::error::Error for DeserializeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    //    self.inner.source()
    todo!()
  }
}

impl Trace for DeserializeError {
  fn trace<R>(self, _trace: R) -> Self
  where
    R: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
  {
    todo!()
    //    Self::RkyvError()
    //      inner: self.inner.trace(trace),
    //    }
  }
}

impl Source for DeserializeError {
  fn new<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
    Self::RkyvError(BoxedError::new(source))
  }
}

pub type CacheableValidator<'a> = HighValidator<'a, DeserializeError>;
pub type CacheableDeserializer = Strategy<Pool, DeserializeError>;

/*pub fn get_deserializer_context<'a, C: Any>(
  deserializer: &'a mut CacheableDeserializer,
) -> Option<&'a C> {
  match deserializer.start_pooling(CONTEXT_ADDR) {
    PoolingState::Finished(ptr) => {
      let ctx = unsafe { &*(ptr.data_address()) as &dyn Any };
      ctx.downcast_ref::<C>()
    }
    _ => None,
  }
}*/

pub fn from_bytes<T, C: Any>(bytes: &[u8], context: &C) -> Result<T, DeserializeError>
where
  T: Archive,
  T::Archived: for<'a> CheckBytes<CacheableValidator<'a>> + Deserialize<T, CacheableDeserializer>,
{
  let mut deserializer = Pool::default();
  unsafe {
    let ctx_ptr = ErasedPtr::new(NonNull::new_unchecked(context as *const _ as *mut ()));
    Pooling::<DeserializeError>::start_pooling(&mut deserializer, CONTEXT_ADDR);
    Pooling::<DeserializeError>::finish_pooling(
      &mut deserializer,
      CONTEXT_ADDR,
      ctx_ptr,
      default_drop,
    )?;
  }
  deserialize_using(
    access::<T::Archived, DeserializeError>(bytes)?,
    &mut deserializer,
  )
}
