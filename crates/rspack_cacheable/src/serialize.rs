use std::any::Any;

use rkyv::{
  api::{high::HighSerializer, serialize_using},
  rancor::{BoxedError, Source, Trace},
  ser::{
    allocator::{Arena, ArenaHandle},
    sharing::Share,
    Serializer, Sharing,
  },
  util::AlignedVec,
  Serialize,
};

const CONTEXT_ADDR: usize = 1;

#[derive(Debug)]
pub enum SerializeError {
  RkyvError(BoxedError),
  // A serialize failed occurred
  SerializeFailed(&'static str),
}

impl std::fmt::Display for SerializeError {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //    write!(f, "{}", self.inner)?;
    Ok(())
  }
}

impl std::error::Error for SerializeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    //    self.inner.source()
    todo!()
  }
}

impl Trace for SerializeError {
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

impl Source for SerializeError {
  fn new<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
    Self::RkyvError(BoxedError::new(source))
  }
}

pub type CacheableSerializer<'a> = HighSerializer<AlignedVec, ArenaHandle<'a>, SerializeError>;

/*pub fn get_serializer_context<'a, C: Any>(
  serializer: &'a mut CacheableSerializer<'a>,
) -> Option<&'a C> {
  match serializer.start_sharing(CONTEXT_ADDR) {
    SharingState::Finished(addr) => {
      let ctx = unsafe { &*(addr as *const ()) as &dyn Any };
      ctx.downcast_ref::<C>()
    }
    _ => None,
  }
}*/

pub fn to_bytes<T, C: Any>(value: &T, ctx: &C) -> Result<Vec<u8>, SerializeError>
where
  T: for<'a> Serialize<CacheableSerializer<'a>>,
{
  let mut arena = Arena::new();
  let mut serializer = Serializer::new(AlignedVec::new(), arena.acquire(), Share::new());
  Sharing::<SerializeError>::start_sharing(&mut serializer, CONTEXT_ADDR);
  Sharing::<SerializeError>::finish_sharing(
    &mut serializer,
    CONTEXT_ADDR,
    ctx as *const _ as usize,
  )?;
  serialize_using(value, &mut serializer)?;
  Ok(serializer.into_writer().into_vec())
}
