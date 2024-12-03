use rkyv::{
  rancor::Fallible,
  tuple::ArchivedTuple3,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use crate::with::AsCacheable;

pub struct AsTuple3<A = AsCacheable, B = AsCacheable, C = AsCacheable> {
  _target: (A, B, C),
}

impl<A, B, C, K, V, H> ArchiveWith<(K, V, H)> for AsTuple3<A, B, C>
where
  A: ArchiveWith<K>,
  B: ArchiveWith<V>,
  C: ArchiveWith<H>,
{
  type Archived = ArchivedTuple3<A::Archived, B::Archived, C::Archived>;
  type Resolver = ArchivedTuple3<A::Resolver, B::Resolver, C::Resolver>;

  #[inline]
  fn resolve_with(field: &(K, V, H), resolver: Self::Resolver, out: Place<Self::Archived>) {
    let field_ptr = unsafe { &raw mut (*out.ptr()).0 };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    A::resolve_with(&field.0, resolver.0, field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    B::resolve_with(&field.1, resolver.1, field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).2 };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    C::resolve_with(&field.2, resolver.2, field_out);
  }
}

impl<A, B, C, K, V, H, S> SerializeWith<(K, V, H), S> for AsTuple3<A, B, C>
where
  A: SerializeWith<K, S>,
  B: SerializeWith<V, S>,
  C: SerializeWith<H, S>,
  S: Fallible + ?Sized,
{
  #[inline]
  fn serialize_with(field: &(K, V, H), serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok(ArchivedTuple3(
      A::serialize_with(&field.0, serializer)?,
      B::serialize_with(&field.1, serializer)?,
      C::serialize_with(&field.2, serializer)?,
    ))
  }
}

impl<A, B, C, K, V, H, D>
  DeserializeWith<ArchivedTuple3<A::Archived, B::Archived, C::Archived>, (K, V, H), D>
  for AsTuple3<A, B, C>
where
  A: ArchiveWith<K> + DeserializeWith<A::Archived, K, D>,
  B: ArchiveWith<V> + DeserializeWith<B::Archived, V, D>,
  C: ArchiveWith<H> + DeserializeWith<C::Archived, H, D>,
  D: Fallible + ?Sized,
{
  fn deserialize_with(
    field: &ArchivedTuple3<A::Archived, B::Archived, C::Archived>,
    deserializer: &mut D,
  ) -> Result<(K, V, H), D::Error> {
    Ok((
      A::deserialize_with(&field.0, deserializer)?,
      B::deserialize_with(&field.1, deserializer)?,
      C::deserialize_with(&field.2, deserializer)?,
    ))
  }
}
