//! Interning of a header plus a slice, in a single allocation.
//!
//! Ported from rust-analyzer (MIT OR Apache-2.0):
//! <https://github.com/rust-lang/rust-analyzer/blob/baabc5825f3f6640e99fe32887bbeced640f825e/crates/intern/src/intern_slice.rs>
//!
//! Upstream supports GC mode only, so this is the non-GC variant: it carries the same
//! drop-removal protocol as [`crate::Interned`].
//!
//! [`InternedSlice`] is essentially `Interned<(Header, Box<[Item]>)>`, except that there is one
//! allocation rather than two and the handle is a thin pointer. Interning takes the items by
//! reference, so a value that is already interned costs no allocation at all.

use std::{
  fmt::{self, Debug},
  hash::{Hash, Hasher},
  sync::OnceLock,
};

use dashmap::{DashMap, SharedValue};
use rustc_hash::FxBuildHasher;
use triomphe::ThinArc;

type InternMap<T> =
  DashMap<ThinArc<<T as SliceInternable>::Header, <T as SliceInternable>::Item>, (), FxBuildHasher>;

pub struct InternedSlice<T: SliceInternable> {
  arc: ThinArc<T::Header, T::Item>,
}

impl<T: SliceInternable> InternedSlice<T> {
  pub fn new(header: T::Header, items: &[T::Item]) -> Self {
    let storage = T::storage().get();
    let hash = T::hash(&header, items);
    let shard = &storage.shards()[storage.determine_shard(hash as usize)];

    // Values are usually interned already, and rspack interns from many rayon threads at once,
    // so look first under a shared lock to keep them from serializing on the shard. A concurrent
    // `drop_slow` cannot remove the entry while we hold this lock, and if it removed the entry
    // before we took it we simply miss and fall through to the write lock below.
    {
      let shard = shard.read();
      if let Some(bucket) = shard.find(hash, |(other, _)| T::eq(&other.slice, items)) {
        // SAFETY: We just found this bucket and still hold the shard lock.
        return unsafe {
          Self {
            arc: bucket.as_ref().0.clone(),
          }
        };
      }
    }

    let mut shard = shard.write();
    // Atomically look up the value again and insert it if it is still missing, so that two
    // threads interning the same value cannot both insert it.
    let bucket = match shard.find_or_find_insert_slot(
      hash,
      |(other, _)| T::eq(&other.slice, items),
      |(other, _)| T::hash(&other.header.header, &other.slice),
    ) {
      Ok(bucket) => bucket,
      // SAFETY: The slot came from `find_or_find_insert_slot()`, and the table wasn't modified since then.
      Err(insert_slot) => unsafe {
        shard.insert_in_slot(
          hash,
          insert_slot,
          (
            ThinArc::from_header_and_slice(header, items),
            SharedValue::new(()),
          ),
        )
      },
    };
    // SAFETY: We just retrieved/inserted this bucket.
    unsafe {
      Self {
        arc: bucket.as_ref().0.clone(),
      }
    }
  }

  #[inline]
  pub fn header(&self) -> &T::Header {
    &self.arc.header.header
  }

  #[inline]
  pub fn items(&self) -> &[T::Item] {
    &self.arc.slice
  }

  #[cold]
  fn drop_slow(&mut self) {
    let storage = T::storage().get();
    let hash = T::hash(self.header(), self.items());
    let mut shard = storage.shards()[storage.determine_shard(hash as usize)].write();

    if ThinArc::strong_count(&self.arc) != 2 {
      // Another thread has interned another copy.
      return;
    }

    // Identify the entry by address: it is this very allocation, so there is no need to run the
    // item comparison again.
    let this = self.arc.as_ptr();
    shard.remove_entry(hash, |(other, _)| other.as_ptr() == this);

    // Shrink the backing storage if the shard is less than 50% occupied.
    if shard.len() * 2 < shard.capacity() {
      let len = shard.len();
      shard.shrink_to(len, |(other, _)| {
        T::hash(&other.header.header, &other.slice)
      });
    }
  }
}

impl<T: SliceInternable> Drop for InternedSlice<T> {
  #[inline]
  fn drop(&mut self) {
    // When the last handle is dropped, remove the value from the global map.
    if ThinArc::strong_count(&self.arc) == 2 {
      // Only `self` and the global map point to the value.
      self.drop_slow();
    }
  }
}

/// Compares interned values using pointer equality.
impl<T: SliceInternable> PartialEq for InternedSlice<T> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.arc.as_ptr() == other.arc.as_ptr()
  }
}

impl<T: SliceInternable> Eq for InternedSlice<T> {}

impl<T: SliceInternable> Hash for InternedSlice<T> {
  /// `write_u64` rather than `write_usize`, for the reason given on [`crate::Interned`]'s `Hash`.
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.arc.as_ptr().addr() as u64)
  }
}

impl<T: SliceInternable> Clone for InternedSlice<T> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      arc: self.arc.clone(),
    }
  }
}

impl<T> Debug for InternedSlice<T>
where
  T: SliceInternable,
  T::Header: Debug,
  T::Item: Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("InternedSlice")
      .field("header", self.header())
      .field("items", &self.items())
      .finish()
  }
}

pub struct InternSliceStorage<T: SliceInternable> {
  map: OnceLock<InternMap<T>>,
}

#[allow(
  clippy::new_without_default,
  reason = "this a const fn, so it can't be default yet. See <https://github.com/rust-lang/rust/issues/63065>"
)]
impl<T: SliceInternable> InternSliceStorage<T> {
  pub const fn new() -> Self {
    Self {
      map: OnceLock::new(),
    }
  }

  fn get(&self) -> &InternMap<T> {
    self
      .map
      .get_or_init(|| DashMap::with_capacity_and_hasher(1024, FxBuildHasher))
  }
}

pub trait SliceInternable: Sized + Send + Sync + 'static {
  type Header: Eq + Hash + Send + Sync;
  type Item: Copy + Eq + Hash + Send + Sync;

  /// The hash the value is bucketed by. Implementors whose header already caches a hash of the
  /// items can return it directly, which makes interning free of re-hashing.
  fn hash(header: &Self::Header, items: &[Self::Item]) -> u64;

  /// Whether two entries are the same value. Only the items take part, so the header must be a
  /// pure function of them — otherwise interning would keep whichever header arrived first.
  fn eq(a: &[Self::Item], b: &[Self::Item]) -> bool;

  fn storage() -> &'static InternSliceStorage<Self>;
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Bytes;

  impl SliceInternable for Bytes {
    type Header = u64;
    type Item = u8;

    fn hash(header: &u64, _items: &[u8]) -> u64 {
      *header
    }

    fn eq(a: &[u8], b: &[u8]) -> bool {
      a == b
    }

    fn storage() -> &'static InternSliceStorage<Self> {
      static STORAGE: InternSliceStorage<Bytes> = InternSliceStorage::new();
      &STORAGE
    }
  }

  fn intern(items: &[u8]) -> InternedSlice<Bytes> {
    InternedSlice::new(items.len() as u64, items)
  }

  fn map_len() -> usize {
    Bytes::storage().get().len()
  }

  #[test]
  fn smoke_test() {
    let base = map_len();

    let a = intern(b"aa");
    let same_a = intern(b"aa");
    let cloned_a = a.clone();
    let b = intern(b"bb");

    assert_eq!(map_len(), base + 2, "equal values share one entry");
    assert_eq!(a, same_a);
    assert_eq!(a.items(), b"aa");
    assert_eq!(*a.header(), 2);
    assert_ne!(a, b);

    drop(same_a);
    drop(cloned_a);
    assert_eq!(map_len(), base + 2, "still held by `a`");

    drop(a);
    assert_eq!(map_len(), base + 1);
    drop(b);
    assert_eq!(map_len(), base);
  }

  #[test]
  fn interning_races_with_dropping() {
    // A hash that ignores the length, so unequal values collide and exercise the item comparison.
    struct Colliding;

    impl SliceInternable for Colliding {
      type Header = u64;
      type Item = u8;

      fn hash(_header: &u64, _items: &[u8]) -> u64 {
        0
      }

      fn eq(a: &[u8], b: &[u8]) -> bool {
        a == b
      }

      fn storage() -> &'static InternSliceStorage<Self> {
        static STORAGE: InternSliceStorage<Colliding> = InternSliceStorage::new();
        &STORAGE
      }
    }

    let storage = Colliding::storage().get();
    assert_eq!(storage.len(), 0);

    std::thread::scope(|scope| {
      for _ in 0..8 {
        scope.spawn(|| {
          for i in 0..2000u32 {
            let items = [(i % 16) as u8, 7];
            let value = InternedSlice::<Colliding>::new(0, &items);
            assert_eq!(value.items(), items);
          }
        });
      }
    });

    assert_eq!(storage.len(), 0, "every value is freed once unreferenced");
  }
}
