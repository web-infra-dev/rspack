use rkyv::{
  rancor::Fallible,
  with::{ArchiveWith, SerializeWith},
  Place,
};

use crate::with::AsCacheable;

pub struct Inline<T = AsCacheable> {
  _inner: T,
}

impl<'a, T, F> ArchiveWith<&'a F> for Inline<T>
where
  T: ArchiveWith<F>,
{
  type Archived = T::Archived;
  type Resolver = T::Resolver;

  #[inline]
  fn resolve_with(field: &&F, resolver: Self::Resolver, out: Place<Self::Archived>) {
    T::resolve_with(field, resolver, out)
  }
}

impl<'a, T, F, S> SerializeWith<&'a F, S> for Inline<T>
where
  T: SerializeWith<F, S>,
  S: ?Sized + Fallible,
{
  #[inline]
  fn serialize_with(field: &&F, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    T::serialize_with(field, serializer)
  }
}
