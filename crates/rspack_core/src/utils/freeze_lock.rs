use std::{fmt, ops::Deref, sync::OnceLock};

use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
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
/// Inspired by [rustc's `FreezeLock`], adapted for shared, cacheable metadata.
///
/// Building owns a `UniqueArc`; freezing converts it to an `Arc` without moving
/// the value. Reads after publication do not acquire the build lock. Restored
/// cache values are already frozen and cannot be made mutable again.
///
/// [rustc's `FreezeLock`]: https://github.com/rust-lang/rust/blob/main/compiler/rustc_data_structures/src/sync/freeze.rs
pub struct FreezeLock<T> {
  frozen: OnceLock<Arc<T>>,
  building: RwLock<Option<UniqueArc<T>>>,
}

impl<T> FreezeLock<T> {
  pub fn new(value: T) -> Self {
    Self {
      frozen: OnceLock::new(),
      building: RwLock::new(Some(UniqueArc::new(value))),
    }
  }

  pub fn read(&self) -> FreezeReadGuard<'_, T> {
    if let Some(value) = self.frozen.get() {
      return FreezeReadGuard(FreezeRead::Frozen(value));
    }
    let building = self.building.read();
    // Publication may have completed while we were waiting for the lock.
    if let Some(value) = self.frozen.get() {
      FreezeReadGuard(FreezeRead::Frozen(value))
    } else {
      FreezeReadGuard(FreezeRead::Building(RwLockReadGuard::map(
        building,
        |value| &**value.as_ref().expect("metadata must be initialized"),
      )))
    }
  }

  pub fn frozen(&self) -> Option<&T> {
    self.frozen.get().map(|value| &**value)
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
  pub(crate) fn update(&self, update: impl FnOnce(&mut T)) {
    let mut building = self.building.write();
    update(building.as_mut().expect("metadata is already frozen"));
  }

  pub fn freeze(&self) -> &Arc<T> {
    if let Some(value) = self.frozen.get() {
      return value;
    }
    let mut building = self.building.write();
    self.frozen.get_or_init(|| {
      building
        .take()
        .expect("metadata must be initialized")
        .shareable()
    })
  }

  /// Publish metadata retained from a successful build after a failed rebuild.
  pub(crate) fn freeze_with(&self, value: Arc<T>) {
    let mut building = self.building.write();
    assert!(self.frozen.set(value).is_ok(), "metadata is already frozen");
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
      frozen: OnceLock::from(value),
      building: RwLock::new(None),
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
  Building(MappedRwLockReadGuard<'a, T>),
  Frozen(&'a T),
}

impl<'a, T: ?Sized> FreezeReadGuard<'a, T> {
  pub fn map<U: ?Sized>(self, map: impl FnOnce(&T) -> &U) -> FreezeReadGuard<'a, U> {
    FreezeReadGuard(match self.0 {
      FreezeRead::Building(value) => FreezeRead::Building(MappedRwLockReadGuard::map(value, map)),
      FreezeRead::Frozen(value) => FreezeRead::Frozen(map(value)),
    })
  }

  pub fn try_map<U: ?Sized>(
    self,
    map: impl FnOnce(&T) -> Option<&U>,
  ) -> Option<FreezeReadGuard<'a, U>> {
    Some(FreezeReadGuard(match self.0 {
      FreezeRead::Building(value) => {
        FreezeRead::Building(MappedRwLockReadGuard::try_map(value, map).ok()?)
      }
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
      .frozen
      .get()
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
      .frozen
      .get()
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
