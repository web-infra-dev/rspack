use rkyv::{
  Serialize,
  api::{high::HighSerializer, serialize_using},
  ser::{
    Serializer as RkyvSerializer,
    allocator::{Arena, ArenaHandle},
    sharing::Share,
  },
  util::AlignedVec,
};

use crate::{
  context::{CacheableContext, ContextGuard},
  error::{Error, Result},
};

pub type Serializer<'a> = HighSerializer<AlignedVec, ArenaHandle<'a>, Error>;

/// Transform struct to bytes
///
/// This function implementation refers to rkyv::to_bytes and
/// add custom error and context support
pub fn to_bytes<T, C: CacheableContext>(value: &T, ctx: &C) -> Result<Vec<u8>>
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
