use rkyv::{
  rancor::Fallible,
  tuple::ArchivedTuple2,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use crate::with::AsCacheable;

pub struct AsTuple2<A = AsCacheable, B = AsCacheable> {
  _target: (A, B),
}

impl<A, B, K, V> ArchiveWith<(K, V)> for AsTuple2<A, B>
where
  A: ArchiveWith<K>,
  B: ArchiveWith<V>,
{
  type Archived = ArchivedTuple2<A::Archived, B::Archived>;
  type Resolver = ArchivedTuple2<A::Resolver, B::Resolver>;

  #[inline]
  fn resolve_with(field: &(K, V), resolver: Self::Resolver, out: Place<Self::Archived>) {
    let field_ptr = unsafe { &raw mut (*out.ptr()).0 };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    A::resolve_with(&field.0, resolver.0, field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).1 };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    B::resolve_with(&field.1, resolver.1, field_out);
  }
}

impl<A, B, K, V, S> SerializeWith<(K, V), S> for AsTuple2<A, B>
where
  A: SerializeWith<K, S>,
  B: SerializeWith<V, S>,
  S: Fallible + ?Sized,
{
  #[inline]
  fn serialize_with(field: &(K, V), serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok(ArchivedTuple2(
      A::serialize_with(&field.0, serializer)?,
      B::serialize_with(&field.1, serializer)?,
    ))
  }
}

impl<A, B, K, V, D> DeserializeWith<ArchivedTuple2<A::Archived, B::Archived>, (K, V), D>
  for AsTuple2<A, B>
where
  A: ArchiveWith<K> + DeserializeWith<A::Archived, K, D>,
  B: ArchiveWith<V> + DeserializeWith<B::Archived, V, D>,
  D: Fallible + ?Sized,
{
  fn deserialize_with(
    field: &ArchivedTuple2<A::Archived, B::Archived>,
    deserializer: &mut D,
  ) -> Result<(K, V), D::Error> {
    Ok((
      A::deserialize_with(&field.0, deserializer)?,
      B::deserialize_with(&field.1, deserializer)?,
    ))
  }
}
