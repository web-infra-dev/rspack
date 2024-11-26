use rkyv::{
  rancor::Fallible,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
};

// reference: rkyv::with::Identity
pub struct AsCacheable;

impl<T: Archive> ArchiveWith<T> for AsCacheable {
  type Archived = Archived<T>;
  type Resolver = Resolver<T>;

  #[inline]
  fn resolve_with(field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    field.resolve(resolver, out)
  }
}

impl<T, S> SerializeWith<T, S> for AsCacheable
where
  T: Archive + Serialize<S>,
  S: Fallible + ?Sized,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    field.serialize(serializer)
  }
}

impl<T, D> DeserializeWith<Archived<T>, T, D> for AsCacheable
where
  T: Archive,
  T::Archived: Deserialize<T, D>,
  D: Fallible + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &Archived<T>, de: &mut D) -> Result<T, D::Error> {
    T::Archived::deserialize(field, de)
  }
}
