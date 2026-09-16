use std::{num::NonZeroU32, sync::atomic::AtomicU32};

#[cfg(allocative)]
use rspack_util::allocative;

use crate::ChunkGroup;

// This allocator deliberately lives outside graph snapshots. Cloned graphs must
// never allocate the same identity when they reuse the same local slot.
static NEXT_CHUNK_UKEY: AtomicU32 = AtomicU32::new(1);

/// An in-process Chunk identity and its graph-local storage slot.
/// Ordering preserves allocation order, independently of slot reuse.
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkUkey {
  identity: NonZeroU32,
  index: u32,
}

impl ChunkUkey {
  pub(crate) fn allocate(index: u32) -> rspack_error::Result<Self> {
    let identity = NEXT_CHUNK_UKEY
      .fetch_update(
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
        |value| value.checked_add(1),
      )
      .map_err(|_| rspack_error::error!("Chunk identity space exhausted"))?;
    Ok(Self {
      identity: NonZeroU32::new(identity).expect("Chunk identities start at one"),
      index,
    })
  }

  pub(crate) fn index(self) -> usize {
    self.index as usize
  }

  /// Legacy identity projection for diagnostic tooling; never a storage index.
  pub fn as_u32(&self) -> u32 {
    self.identity.get()
  }

  /// Lossless process-local representation. Not a persistent cache identity.
  pub fn as_u64(self) -> u64 {
    ((self.identity.get() as u64) << 32) | self.index as u64
  }

  /// Decode a process-local diagnostic reference, rejecting the vacant identity.
  pub fn from_u64(value: u64) -> Option<Self> {
    Some(Self {
      identity: NonZeroU32::new((value >> 32) as u32)?,
      index: value as u32,
    })
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
