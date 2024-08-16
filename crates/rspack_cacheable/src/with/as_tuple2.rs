use rkyv::{
  collections::util::validation::ArchivedEntryError,
  out_field,
  validation::ArchiveContext,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  CheckBytes, Fallible,
};

use crate::{with::AsCacheable, DeserializeError};

pub struct Tuple2<A, B> {
  a: A,
  b: B,
}

pub struct AsTuple2<A = AsCacheable, B = AsCacheable> {
  _target: (A, B),
}

impl<A, B, K, V> ArchiveWith<(K, V)> for AsTuple2<A, B>
where
  A: ArchiveWith<K>,
  B: ArchiveWith<V>,
{
  type Archived = Tuple2<A::Archived, B::Archived>;
  type Resolver = Tuple2<A::Resolver, B::Resolver>;

  #[inline]
  unsafe fn resolve_with(
    field: &(K, V),
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let (fp, fo) = out_field!(out.a);
    A::resolve_with(&field.0, pos + fp, resolver.a, fo);

    let (fp, fo) = out_field!(out.b);
    B::resolve_with(&field.1, pos + fp, resolver.b, fo);
  }
}

impl<A, B, K, V, S: Fallible + ?Sized> SerializeWith<(K, V), S> for AsTuple2<A, B>
where
  A: SerializeWith<K, S>,
  B: SerializeWith<V, S>,
{
  #[inline]
  fn serialize_with(field: &(K, V), serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok(Tuple2 {
      a: A::serialize_with(&field.0, serializer)?,
      b: B::serialize_with(&field.1, serializer)?,
    })
  }
}

impl<A, B, C> CheckBytes<C> for Tuple2<A, B>
where
  A: CheckBytes<C>,
  B: CheckBytes<C>,
  C: ArchiveContext + ?Sized,
{
  type Error = DeserializeError;

  #[inline]
  unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
    A::check_bytes(core::ptr::addr_of!((*value).a), context)
      .map_err(|_| DeserializeError::CheckBytesError)?;
    B::check_bytes(core::ptr::addr_of!((*value).b), context)
      .map_err(|_| DeserializeError::CheckBytesError)?;
    Ok(&*value)
  }
}

impl<A, B, K, V, D> DeserializeWith<Tuple2<A::Archived, B::Archived>, (K, V), D> for AsTuple2<A, B>
where
  A: ArchiveWith<K> + DeserializeWith<A::Archived, K, D>,
  B: ArchiveWith<V> + DeserializeWith<B::Archived, V, D>,
  D: ?Sized + Fallible,
{
  fn deserialize_with(
    field: &Tuple2<A::Archived, B::Archived>,
    deserializer: &mut D,
  ) -> Result<(K, V), D::Error> {
    Ok((
      A::deserialize_with(&field.a, deserializer)?,
      B::deserialize_with(&field.b, deserializer)?,
    ))
  }
}
