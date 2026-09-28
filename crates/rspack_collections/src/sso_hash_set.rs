use std::{borrow::Borrow, collections::hash_set, hash::Hash, slice};

use either::Either;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

const INLINE_CAPACITY: usize = 4;

/// A set that stores up to four elements inline, then switches to an `FxHashSet`.
///
/// Iteration order is unspecified. Removal and clearing retain the hash table
/// once allocated, avoiding repeated promotion when a set grows and shrinks.
/// Cloning copies the elements to provide independent snapshots, such as module
/// chunk memberships retained while mutating the chunk graph. Cloning inline
/// storage does not allocate storage for the set.
#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SsoHashSet<T>(Storage<T>);

#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
enum Storage<T> {
  Inline(SmallVec<[T; INLINE_CAPACITY]>),
  Heap(FxHashSet<T>),
}

impl<T> Default for SsoHashSet<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> SsoHashSet<T> {
  pub fn new() -> Self {
    Self(Storage::Inline(SmallVec::new()))
  }

  #[inline]
  pub fn len(&self) -> usize {
    match &self.0 {
      Storage::Inline(values) => values.len(),
      Storage::Heap(values) => values.len(),
    }
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn clear(&mut self) {
    match &mut self.0 {
      Storage::Inline(values) => values.clear(),
      Storage::Heap(values) => values.clear(),
    }
  }

  #[inline]
  pub fn iter(&self) -> Either<slice::Iter<'_, T>, hash_set::Iter<'_, T>> {
    match &self.0 {
      Storage::Inline(values) => Either::Left(values.iter()),
      Storage::Heap(values) => Either::Right(values.iter()),
    }
  }
}

impl<T: Eq + Hash> SsoHashSet<T> {
  #[inline]
  pub fn contains<Q: ?Sized + Eq + Hash>(&self, value: &Q) -> bool
  where
    T: Borrow<Q>,
  {
    match &self.0 {
      Storage::Inline(values) => values.iter().any(|item| item.borrow() == value),
      Storage::Heap(values) => values.contains(value),
    }
  }

  #[inline]
  pub fn insert(&mut self, value: T) -> bool {
    match &mut self.0 {
      Storage::Inline(values) => {
        if values.contains(&value) {
          return false;
        }
        if values.len() < INLINE_CAPACITY {
          values.push(value);
        } else {
          let mut set =
            FxHashSet::with_capacity_and_hasher(INLINE_CAPACITY + 1, Default::default());
          set.extend(values.drain(..));
          set.insert(value);
          self.0 = Storage::Heap(set);
        }
        true
      }
      Storage::Heap(values) => values.insert(value),
    }
  }

  #[inline]
  pub fn remove<Q: ?Sized + Eq + Hash>(&mut self, value: &Q) -> bool
  where
    T: Borrow<Q>,
  {
    match &mut self.0 {
      Storage::Inline(values) => {
        if let Some(index) = values.iter().position(|item| item.borrow() == value) {
          values.swap_remove(index);
          true
        } else {
          false
        }
      }
      Storage::Heap(values) => values.remove(value),
    }
  }

  pub fn is_subset(&self, other: &Self) -> bool {
    self.len() <= other.len() && self.iter().all(|value| other.contains(value))
  }
}

impl<T: Eq + Hash> Extend<T> for SsoHashSet<T> {
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    for value in iter {
      self.insert(value);
    }
  }
}

impl<T: Eq + Hash> FromIterator<T> for SsoHashSet<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut set = Self::new();
    set.extend(iter);
    set
  }
}

impl<'a, T> IntoIterator for &'a SsoHashSet<T> {
  type Item = &'a T;
  type IntoIter = Either<slice::Iter<'a, T>, hash_set::Iter<'a, T>>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<T> IntoIterator for SsoHashSet<T> {
  type Item = T;
  type IntoIter = Either<smallvec::IntoIter<[T; INLINE_CAPACITY]>, hash_set::IntoIter<T>>;

  fn into_iter(self) -> Self::IntoIter {
    match self.0 {
      Storage::Inline(values) => Either::Left(values.into_iter()),
      Storage::Heap(values) => Either::Right(values.into_iter()),
    }
  }
}
