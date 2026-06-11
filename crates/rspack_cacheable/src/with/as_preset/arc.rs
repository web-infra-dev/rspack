use std::sync::Arc;

use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  rancor::Fallible,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;

impl<T> ArchiveWith<Arc<T>> for AsPreset
where
  Arc<T>: Archive,
  T: ?Sized,
{
  type Archived = Archived<Arc<T>>;
  type Resolver = Resolver<Arc<T>>;

  #[inline]
  fn resolve_with(field: &Arc<T>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    Archive::resolve(field, resolver, out);
  }
}

impl<T, S> SerializeWith<Arc<T>, S> for AsPreset
where
  Arc<T>: Archive + Serialize<S>,
  S: Fallible + ?Sized,
  T: ?Sized,
{
  #[inline]
  fn serialize_with(field: &Arc<T>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Serialize::serialize(field, serializer)
  }
}

impl<T, D> DeserializeWith<Archived<Arc<T>>, Arc<T>, D> for AsPreset
where
  Arc<T>: Archive,
  Archived<Arc<T>>: Deserialize<Arc<T>, D>,
  D: Fallible + ?Sized,
  T: ?Sized,
{
  #[inline]
  fn deserialize_with(field: &Archived<Arc<T>>, deserializer: &mut D) -> Result<Arc<T>, D::Error> {
    Deserialize::deserialize(field, deserializer)
  }
}
