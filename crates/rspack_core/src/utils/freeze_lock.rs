use std::{
  cell::UnsafeCell,
  fmt,
  ops::Deref,
  sync::atomic::{AtomicBool, Ordering},
};

use atomic_refcell::{AtomicRef, AtomicRefCell};
use rspack_cacheable::{
  Error,
  rkyv::{
    Archive, Deserialize, Place, Serialize,
    rancor::Fallible,
    rc::{ArchivedRc, Flavor},
  },
};
use triomphe::{Arc, UniqueArc};

/// Metadata with independent build and publication lifetimes.
///
/// Like [Servo's shared lock], build access uses atomic borrow checking rather
/// than a blocking lock: multiple readers or one writer, with conflicting access
/// panicking. `get` retains the read guard internally; `get_mut` uses an exclusive
/// Rust borrow, and field-specific updates retain a write guard internally.
///
/// Freezing requires exclusive build access and converts the `UniqueArc` to an
/// `Arc` without moving the value. Published reads only acquire-load a flag and
/// borrow the immutable value. Cached values are restored already frozen.
///
/// [Servo's shared lock]: https://github.com/mozilla-firefox/firefox/blob/main/servo/components/style/shared_lock.rs
pub struct FreezeLock<T> {
  published: AtomicBool,
  frozen: UnsafeCell<Option<Arc<T>>>,
  building: AtomicRefCell<Option<UniqueArc<T>>>,
}

// SAFETY: before publication, AtomicRefCell enforces exclusive writes and shared
// reads. The frozen slot is written only while holding the exclusive build
// borrow, and never changed after the release-store to `published`. Readers
// acquire-load that flag before accessing the slot. Sharing also requires T to
// be Send because publication and field-specific updates can occur on any thread.
unsafe impl<T: Send + Sync> Sync for FreezeLock<T> {}

impl<T> FreezeLock<T> {
  pub fn new(value: T) -> Self {
    Self {
      published: AtomicBool::new(false),
      frozen: UnsafeCell::new(None),
      building: AtomicRefCell::new(Some(UniqueArc::new(value))),
    }
  }

  /// Read metadata, retaining a shared build borrow until the guard is dropped.
  /// Panics if a build update or publication is in progress.
  pub fn get(&self) -> FreezeReadGuard<'_, T> {
    if let Some(value) = self.frozen_arc() {
      return FreezeReadGuard(FreezeRead::Frozen(value));
    }
    let building = self.building.borrow();
    // Publication may have completed before we acquired the build borrow.
    if let Some(value) = self.frozen_arc() {
      FreezeReadGuard(FreezeRead::Frozen(value))
    } else {
      FreezeReadGuard(FreezeRead::Building(AtomicRef::map(building, |value| {
        &**value.as_ref().expect("metadata must be initialized")
      })))
    }
  }

  pub fn read(&self) -> FreezeReadGuard<'_, T> {
    self.get()
  }

  /// Read published metadata without a build borrow guard.
  ///
  /// Only build borrow checking is skipped: publication is checked in all builds.
  /// Panics if the metadata has not been frozen yet.
  #[inline]
  pub fn get_unchecked(&self) -> &T {
    self
      .frozen()
      .expect("metadata must be frozen before unchecked access")
  }

  fn frozen_arc(&self) -> Option<&Arc<T>> {
    if !self.published.load(Ordering::Acquire) {
      return None;
    }
    // SAFETY: the acquire-load observes initialization of the frozen slot, which
    // is never modified again. Its value remains alive for the lifetime of self.
    unsafe { &*self.frozen.get() }.as_ref()
  }

  pub fn frozen(&self) -> Option<&T> {
    self.frozen_arc().map(|value| &**value)
  }

  /// Exclusive access while constructing the containing module.
  pub fn get_mut(&mut self) -> &mut T {
    self
      .building
      .get_mut()
      .as_mut()
      .expect("metadata is already frozen")
  }

  /// Used only by field-specific APIs for metadata finalized after module build.
  /// Panics on conflicting borrows or after publication.
  pub(crate) fn update(&self, update: impl FnOnce(&mut T)) {
    let mut building = self.building.borrow_mut();
    update(building.as_mut().expect("metadata is already frozen"));
  }

  /// Publish the value permanently. All build guards must have been dropped.
  pub fn freeze(&self) -> &Arc<T> {
    if let Some(value) = self.frozen_arc() {
      return value;
    }
    let mut building = self.building.borrow_mut();
    // Another publisher may have finished before the exclusive borrow.
    if let Some(value) = self.frozen_arc() {
      return value;
    }
    let value = building
      .take()
      .expect("metadata must be initialized")
      .shareable();
    // SAFETY: the exclusive build borrow excludes other publishers, and the
    // flag is still false, so no reader can access the frozen slot yet.
    unsafe { *self.frozen.get() = Some(value) };
    self.published.store(true, Ordering::Release);
    self.frozen_arc().expect("metadata must be frozen")
  }

  /// Publish metadata retained from a successful build after a failed rebuild.
  pub(crate) fn freeze_with(&self, value: Arc<T>) {
    let mut building = self.building.borrow_mut();
    assert!(self.frozen_arc().is_none(), "metadata is already frozen");
    // SAFETY: as in freeze, the exclusive build borrow excludes publishers and
    // the unpublished slot cannot yet be accessed by readers.
    unsafe { *self.frozen.get() = Some(value) };
    self.published.store(true, Ordering::Release);
    building.take();
  }
}

impl<T: Default> Default for FreezeLock<T> {
  fn default() -> Self {
    Self::new(T::default())
  }
}

impl<T> From<T> for FreezeLock<T> {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

impl<T> From<Arc<T>> for FreezeLock<T> {
  fn from(value: Arc<T>) -> Self {
    Self {
      published: AtomicBool::new(true),
      frozen: UnsafeCell::new(Some(value)),
      building: AtomicRefCell::new(None),
    }
  }
}

impl<T: fmt::Debug> fmt::Debug for FreezeLock<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.read().fmt(f)
  }
}

pub struct FreezeReadGuard<'a, T: ?Sized>(FreezeRead<'a, T>);

enum FreezeRead<'a, T: ?Sized> {
  Building(AtomicRef<'a, T>),
  Frozen(&'a T),
}

impl<'a, T: ?Sized> FreezeReadGuard<'a, T> {
  pub fn map<U: ?Sized>(self, map: impl FnOnce(&T) -> &U) -> FreezeReadGuard<'a, U> {
    FreezeReadGuard(match self.0 {
      FreezeRead::Building(value) => FreezeRead::Building(AtomicRef::map(value, map)),
      FreezeRead::Frozen(value) => FreezeRead::Frozen(map(value)),
    })
  }

  pub fn try_map<U: ?Sized>(
    self,
    map: impl FnOnce(&T) -> Option<&U>,
  ) -> Option<FreezeReadGuard<'a, U>> {
    Some(FreezeReadGuard(match self.0 {
      FreezeRead::Building(value) => FreezeRead::Building(AtomicRef::filter_map(value, map)?),
      FreezeRead::Frozen(value) => FreezeRead::Frozen(map(value)?),
    }))
  }
}

impl<T: ?Sized> Deref for FreezeReadGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    match &self.0 {
      FreezeRead::Building(value) => value,
      FreezeRead::Frozen(value) => value,
    }
  }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for FreezeReadGuard<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.deref().fmt(f)
  }
}

impl<T: Archive> Archive for FreezeLock<T> {
  type Archived = <Arc<T> as Archive>::Archived;
  type Resolver = <Arc<T> as Archive>::Resolver;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    self
      .frozen_arc()
      .expect("metadata must be frozen before caching")
      .resolve(resolver, out);
  }
}

impl<T: Archive, S> Serialize<S> for FreezeLock<T>
where
  Arc<T>: Serialize<S>,
  S: Fallible<Error = Error> + ?Sized,
{
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    self
      .frozen_arc()
      .ok_or(Error::MessageError(
        "metadata must be frozen before caching",
      ))?
      .serialize(serializer)
  }
}

impl<T: Archive, F: Flavor, D> Deserialize<FreezeLock<T>, D> for ArchivedRc<T::Archived, F>
where
  Self: Deserialize<Arc<T>, D>,
  D: Fallible + ?Sized,
{
  fn deserialize(&self, deserializer: &mut D) -> Result<FreezeLock<T>, D::Error> {
    Ok(<Self as Deserialize<Arc<T>, D>>::deserialize(self, deserializer)?.into())
  }
}
