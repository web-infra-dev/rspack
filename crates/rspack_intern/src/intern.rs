//! Interning of single sized values.
//!
//! Ported from rust-analyzer (MIT OR Apache-2.0):
//! <https://github.com/rust-lang/rust-analyzer/blob/baabc5825f3f6640e99fe32887bbeced640f825e/crates/intern/src/intern.rs>
//!
//! Trimmed to the non-GC mode, without `InternedRef`, and using `std::sync::Arc` instead of
//! `triomphe::Arc`.
//!
//! Interning a value returns an [`Interned`] handle. Equal values share one allocation, so
//! equality and hashing are pointer operations. When the last [`Interned`] of a value is
//! dropped the value is freed; interning it again may place it somewhere else.
//!
//! For a value that is a header plus a slice, [`crate::InternedSlice`] stores both in one
//! allocation behind a thin pointer.

use std::{
  fmt::{self, Debug, Display},
  hash::{BuildHasher, Hash, Hasher},
  ops::Deref,
  sync::{Arc, OnceLock},
};

use dashmap::{DashMap, SharedValue};
use rustc_hash::FxBuildHasher;

type InternMap<T> = DashMap<Arc<T>, (), FxBuildHasher>;

pub struct Interned<T: Internable> {
  arc: Arc<T>,
}

impl<T: Internable> Interned<T> {
  pub fn new(obj: T) -> Self {
    let storage = T::storage().get();
    let hash = storage.hasher().hash_one(&obj);
    let shard = &storage.shards()[storage.determine_shard(hash as usize)];

    // Values are usually interned already, and rspack interns from many rayon threads at once,
    // so look first under a shared lock to keep them from serializing on the shard. A concurrent
    // `drop_slow` cannot remove the entry while we hold this lock, and if it removed the entry
    // before we took it we simply miss and fall through to the write lock below.
    {
      let shard = shard.read();
      if let Some(bucket) = shard.find(hash, |(other, _)| **other == obj) {
        // SAFETY: We just found this bucket and still hold the shard lock.
        return unsafe {
          Self {
            arc: bucket.as_ref().0.clone(),
          }
        };
      }
    }

    let mut shard = shard.write();
    // Atomically,
    // - check if `obj` is already in the map
    //   - if so, clone its `Arc` and return it
    //   - if not, box it up, insert it, and return a clone
    // This needs to be atomic (locking the shard) to avoid races with other thread, which could
    // insert the same object between us looking it up and inserting it.
    let bucket = match shard.find_or_find_insert_slot(
      hash,
      |(other, _)| **other == obj,
      |(other, _)| storage.hasher().hash_one(other),
    ) {
      Ok(bucket) => bucket,
      // SAFETY: The slot came from `find_or_find_insert_slot()`, and the table wasn't modified since then.
      Err(insert_slot) => unsafe {
        shard.insert_in_slot(hash, insert_slot, (Arc::new(obj), SharedValue::new(())))
      },
    };
    // SAFETY: We just retrieved/inserted this bucket.
    unsafe {
      Self {
        arc: bucket.as_ref().0.clone(),
      }
    }
  }

  #[cold]
  fn drop_slow(&mut self) {
    let storage = T::storage().get();
    let hash = storage.hasher().hash_one(&self.arc);
    let mut shard = storage.shards()[storage.determine_shard(hash as usize)].write();

    if Arc::strong_count(&self.arc) != 2 {
      // Another thread has interned another copy.
      return;
    }

    shard.remove_entry(hash, |(other, _)| **other == **self);

    // Shrink the backing storage if the shard is less than 50% occupied.
    if shard.len() * 2 < shard.capacity() {
      let len = shard.len();
      shard.shrink_to(len, |(other, _)| storage.hasher().hash_one(other));
    }
  }
}

impl<T: Internable> Drop for Interned<T> {
  #[inline]
  fn drop(&mut self) {
    // When the last handle is dropped, remove the object from the global map.
    if Arc::strong_count(&self.arc) == 2 {
      // Only `self` and the global map point to the object.
      self.drop_slow();
    }
  }
}

/// Compares interned values using pointer equality.
impl<T: Internable> PartialEq for Interned<T> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.arc, &other.arc)
  }
}

impl<T: Internable> Eq for Interned<T> {}

impl<T: Internable> Hash for Interned<T> {
  /// `write_u64` rather than `write_usize`: `ustr::IdentityHasher` only keeps 8-byte writes, and
  /// `write_usize` is 4 bytes on wasm32, which would hash every value to the same bucket.
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(Arc::as_ptr(&self.arc).addr() as u64)
  }
}

impl<T: Internable> AsRef<T> for Interned<T> {
  #[inline]
  fn as_ref(&self) -> &T {
    self
  }
}

impl<T: Internable> Deref for Interned<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.arc
  }
}

impl<T: Internable> Clone for Interned<T> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      arc: self.arc.clone(),
    }
  }
}

impl<T: Debug + Internable> Debug for Interned<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <T as Debug>::fmt(self, f)
  }
}

impl<T: Display + Internable> Display for Interned<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    <T as Display>::fmt(self, f)
  }
}

pub struct InternStorage<T: ?Sized> {
  map: OnceLock<InternMap<T>>,
}

#[allow(
  clippy::new_without_default,
  reason = "this a const fn, so it can't be default yet. See <https://github.com/rust-lang/rust/issues/63065>"
)]
impl<T: ?Sized> InternStorage<T> {
  pub const fn new() -> Self {
    Self {
      map: OnceLock::new(),
    }
  }
}

impl<T: Internable + ?Sized> InternStorage<T> {
  fn get(&self) -> &InternMap<T> {
    self
      .map
      .get_or_init(|| DashMap::with_capacity_and_hasher(1024, FxBuildHasher))
  }
}

pub trait Internable: Hash + Eq + Send + Sync + 'static {
  fn storage() -> &'static InternStorage<Self>;
}

/// Implements `Internable` for a given list of types, making them usable with `Interned`.
#[macro_export]
#[doc(hidden)]
macro_rules! _impl_internable {
  ( $($t:ty),+ $(,)? ) => { $(
    impl $crate::Internable for $t {
      fn storage() -> &'static $crate::InternStorage<Self> {
        static STORAGE: $crate::InternStorage<$t> = $crate::InternStorage::new();
        &STORAGE
      }
    }
  )+ };
}
pub use crate::_impl_internable as impl_internable;

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(PartialEq, Eq, Hash, Debug)]
  struct Str(String);

  // A dedicated type so this test's map is not observed by the other one.
  #[derive(PartialEq, Eq, Hash, Debug)]
  struct ConcurrentStr(String);

  impl_internable!(Str, ConcurrentStr);

  fn intern(s: &str) -> Interned<Str> {
    Interned::new(Str(s.to_string()))
  }

  fn map_len() -> usize {
    Str::storage().get().len()
  }

  #[test]
  fn smoke_test() {
    let base = map_len();

    let a = intern("/a");
    let same_a = intern("/a");
    let cloned_a = a.clone();
    let b = intern("/b");

    assert_eq!(map_len(), base + 2, "equal values share one entry");
    assert_eq!(a, same_a);
    assert!(std::ptr::eq(&*a, &*same_a), "equal values share one Arc");
    assert_ne!(a, b);
    assert_eq!(&*a.0, "/a");

    drop(same_a);
    drop(cloned_a);
    assert_eq!(map_len(), base + 2, "still held by `a`");

    drop(a);
    assert_eq!(map_len(), base + 1);
    drop(b);
    assert_eq!(map_len(), base);

    // Re-interning a freed value works, and hashing stays pointer based.
    let a = intern("/a");
    let same_a = intern("/a");
    let mut hasher = std::hash::DefaultHasher::new();
    a.hash(&mut hasher);
    let a_hash = hasher.finish();
    let mut hasher = std::hash::DefaultHasher::new();
    same_a.hash(&mut hasher);
    assert_eq!(a_hash, hasher.finish());
  }

  #[test]
  fn interning_races_with_dropping() {
    let storage = ConcurrentStr::storage().get();
    assert_eq!(storage.len(), 0);

    std::thread::scope(|scope| {
      for _ in 0..8 {
        scope.spawn(|| {
          for i in 0..2000 {
            let key = format!("{}", i % 16);
            let value = Interned::new(ConcurrentStr(key.clone()));
            assert_eq!(value.0, key);
          }
        });
      }
    });

    assert_eq!(storage.len(), 0, "every value is freed once unreferenced");
  }
}
