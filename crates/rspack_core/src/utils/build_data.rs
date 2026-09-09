use std::fmt;

use rspack_cacheable::{
  Error,
  rkyv::{
    Archive, Deserialize, Place, Serialize,
    rancor::Fallible,
    rc::{ArchivedRc, Flavor},
  },
};
use triomphe::{Arc, UniqueArc};

/// Independently owned metadata while building, shared only after finalization.
/// Transitions require exclusive access to the owning build record.
pub struct BuildData<T>(BuildDataState<T>);

enum BuildDataState<T> {
  Building(Option<UniqueArc<T>>),
  Shared(Arc<T>),
}
impl<T> BuildData<T> {
  pub fn read(&self) -> &T {
    match &self.0 {
      BuildDataState::Building(value) => value.as_deref().expect("build data is initialized"),
      BuildDataState::Shared(value) => value,
    }
  }
  pub fn get_mut(&mut self) -> &mut T {
    match &mut self.0 {
      BuildDataState::Building(value) => value.as_deref_mut().expect("build data is initialized"),
      BuildDataState::Shared(_) => panic!("build data is already finalized"),
    }
  }
  pub fn finish(&mut self) -> &Arc<T> {
    if let BuildDataState::Building(value) = &mut self.0 {
      self.0 = BuildDataState::Shared(value.take().expect("build data is initialized").shareable());
    }
    self.shared().expect("build data is finalized")
  }
  pub fn shared(&self) -> Option<&Arc<T>> {
    match &self.0 {
      BuildDataState::Shared(value) => Some(value),
      BuildDataState::Building(_) => None,
    }
  }
}
impl<T: Default> Default for BuildData<T> {
  fn default() -> Self {
    T::default().into()
  }
}
impl<T> From<T> for BuildData<T> {
  fn from(value: T) -> Self {
    Self(BuildDataState::Building(Some(UniqueArc::new(value))))
  }
}
impl<T> From<Arc<T>> for BuildData<T> {
  fn from(value: Arc<T>) -> Self {
    Self(BuildDataState::Shared(value))
  }
}
impl<T: Archive> Archive for BuildData<T> {
  type Archived = <Arc<T> as Archive>::Archived;
  type Resolver = <Arc<T> as Archive>::Resolver;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    self
      .shared()
      .expect("metadata must be finalized before caching")
      .resolve(resolver, out);
  }
}

impl<T: Archive, S> Serialize<S> for BuildData<T>
where
  Arc<T>: Serialize<S>,
  S: Fallible<Error = Error> + ?Sized,
{
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    self
      .shared()
      .ok_or(Error::MessageError(
        "metadata must be finalized before caching",
      ))?
      .serialize(serializer)
  }
}

impl<T: Archive, F: Flavor, D> Deserialize<BuildData<T>, D> for ArchivedRc<T::Archived, F>
where
  Self: Deserialize<Arc<T>, D>,
  D: Fallible + ?Sized,
{
  fn deserialize(&self, deserializer: &mut D) -> Result<BuildData<T>, D::Error> {
    Ok(<Self as Deserialize<Arc<T>, D>>::deserialize(self, deserializer)?.into())
  }
}

impl<T: fmt::Debug> fmt::Debug for BuildData<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.read().fmt(f)
  }
}
