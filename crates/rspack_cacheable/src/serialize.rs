use std::cell::RefCell;

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
  to_bytes_with_binding(value, ctx).map(|(bytes, _)| bytes)
}

/// Like [`to_bytes`], with paths above the project root stored relative to it. Snapshot keys use
/// it: a key only identifies the path a snapshot validates, and the validation itself happens at
/// the restored location.
pub fn to_bytes_above_root<T, C: CacheableContext>(value: &T, ctx: &C) -> Result<Vec<u8>>
where
  T: for<'a> Serialize<Serializer<'a>>,
{
  let guard = ContextGuard::new(ctx);
  guard.set_allow_above_root(true);
  to_bytes_with_guard(value, guard).map(|(bytes, _)| bytes)
}

pub fn to_bytes_with_binding<T, C: CacheableContext>(value: &T, ctx: &C) -> Result<(Vec<u8>, bool)>
where
  T: for<'a> Serialize<Serializer<'a>>,
{
  to_bytes_with_guard(value, ContextGuard::new(ctx))
}

/// A buffer above this size is not kept for the next value, so one large value doesn't hold its
/// memory on every thread that serialized one.
const MAX_KEPT_CAPACITY: usize = 1 << 20;

thread_local! {
  /// The arena and output buffer of the last value serialized on this thread. A persistent cache
  /// stores many small values, and fresh ones would grow from nothing for each of them.
  static SCRATCH: RefCell<(Arena, AlignedVec)> = RefCell::new((Arena::new(), AlignedVec::new()));
}

fn to_bytes_with_guard<T>(value: &T, guard: ContextGuard<'_>) -> Result<(Vec<u8>, bool)>
where
  T: for<'a> Serialize<Serializer<'a>>,
{
  fn serialize<T>(
    value: &T,
    guard: &ContextGuard<'_>,
    arena: &mut Arena,
    writer: AlignedVec,
  ) -> (AlignedVec, Result<()>)
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    let mut serializer = RkyvSerializer::new(writer, arena.acquire(), Share::new());
    let result = guard
      .add_to_sharing(&mut serializer)
      .and_then(|()| serialize_using(value, &mut serializer).map(|_| ()));
    (serializer.into_writer(), result)
  }

  SCRATCH.with(|scratch| {
    // A value that serializes another while it is being serialized gets buffers of its own.
    let Ok(mut scratch) = scratch.try_borrow_mut() else {
      let (writer, result) = serialize(value, &guard, &mut Arena::new(), AlignedVec::new());
      return result.map(|()| (writer.into_vec(), guard.saw_outside_root()));
    };
    let (arena, writer) = &mut *scratch;
    let mut buffer = std::mem::take(writer);
    buffer.clear();
    let (buffer, result) = serialize(value, &guard, arena, buffer);
    let bytes = result.map(|()| (buffer.as_slice().to_vec(), guard.saw_outside_root()));
    arena.shrink();
    if buffer.capacity() <= MAX_KEPT_CAPACITY {
      *writer = buffer;
    }
    bytes
  })
}
