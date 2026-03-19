use std::sync::atomic::AtomicU32;

#[cfg(allocative)]
use rspack_util::allocative;

use crate::{Chunk, ChunkGroup};

static NEXT_CHUNK_UKEY: AtomicU32 = AtomicU32::new(0);

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkUkey(u32, std::marker::PhantomData<Chunk>);

impl Default for ChunkUkey {
  fn default() -> Self {
    Self::new()
  }
}

impl ChunkUkey {
  pub fn new() -> Self {
    Self(
      NEXT_CHUNK_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
      std::marker::PhantomData,
    )
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

impl From<u32> for ChunkUkey {
  fn from(value: u32) -> Self {
    Self(value, std::marker::PhantomData)
  }
}

static NEXT_CHUNK_GROUP_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ChunkGroupUkey(u32, std::marker::PhantomData<ChunkGroup>);

impl Default for ChunkGroupUkey {
  fn default() -> Self {
    Self::new()
  }
}

impl ChunkGroupUkey {
  pub fn new() -> Self {
    Self(
      NEXT_CHUNK_GROUP_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
      std::marker::PhantomData::default(),
    )
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

impl From<u32> for ChunkGroupUkey {
  fn from(value: u32) -> Self {
    Self(value, std::marker::PhantomData)
  }
}
