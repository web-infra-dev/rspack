use std::any::Any;

use rkyv::{
  api::{high::HighSerializer, serialize_using},
  rancor::{BoxedError, Source, Trace},
  ser::{
    allocator::{Arena, ArenaHandle},
    sharing::Share,
    Serializer as RkyvSerializer,
  },
  util::AlignedVec,
  Serialize,
};

use crate::context::ContextGuard;

#[derive(Debug)]
pub enum SerializeError {
  BoxedError(BoxedError),
  MessageError(&'static str),
  NoContext,
  UnsupportedField,
}

impl std::fmt::Display for SerializeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::BoxedError(error) => error.fmt(f),
      Self::MessageError(msg) => {
        write!(f, "{msg}")
      }
      Self::NoContext => {
        write!(f, "no context")
      }
      Self::UnsupportedField => {
        write!(f, "unsupported field")
      }
    }
  }
}

impl std::error::Error for SerializeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::BoxedError(error) => error.source(),
      _ => None,
    }
  }
}

impl Trace for SerializeError {
  fn trace<R>(self, trace: R) -> Self
  where
    R: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
  {
    Self::BoxedError(BoxedError::trace(BoxedError::new(self), trace))
  }
}

impl Source for SerializeError {
  fn new<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
    Self::BoxedError(BoxedError::new(source))
  }
}

pub type Serializer<'a> = HighSerializer<AlignedVec, ArenaHandle<'a>, SerializeError>;

/// Transform struct to bytes
///
/// This function implementation refers to rkyv::to_bytes and
/// add custom error and context support
pub fn to_bytes<T, C: Any>(value: &T, ctx: &C) -> Result<Vec<u8>, SerializeError>
where
  T: for<'a> Serialize<Serializer<'a>>,
{
  let guard = ContextGuard::new(ctx);
  let mut arena = Arena::new();
  let mut serializer = RkyvSerializer::new(AlignedVec::new(), arena.acquire(), Share::new());
  guard.add_to_sharing(&mut serializer)?;

  serialize_using(value, &mut serializer)?;
  Ok(serializer.into_writer().into_vec())
}
