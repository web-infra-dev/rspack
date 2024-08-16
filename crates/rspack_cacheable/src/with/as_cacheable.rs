use rkyv::{
  ser::{ScratchSpace, Serializer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, Archived, Deserialize, Fallible, Resolver, Serialize,
};

pub struct AsCacheable;

impl<T: Archive> ArchiveWith<T> for AsCacheable {
  type Archived = Archived<T>;
  type Resolver = Resolver<T>;

  #[inline]
  unsafe fn resolve_with(
    field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    field.resolve(pos, resolver, out)
  }
}

impl<T, S> SerializeWith<T, S> for AsCacheable
where
  T: Archive + Serialize<S>,
  S: ?Sized + Serializer + ScratchSpace,
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
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &Archived<T>, de: &mut D) -> Result<T, D::Error> {
    T::Archived::deserialize(field, de)
  }
}
