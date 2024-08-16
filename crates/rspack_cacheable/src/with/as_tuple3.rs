use rkyv::{
  out_field,
  validation::ArchiveContext,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  CheckBytes, Fallible,
};

use crate::{with::AsCacheable, DeserializeError};

pub struct Tuple3<A, B, C> {
  a: A,
  b: B,
  c: C,
}

pub struct AsTuple3<A = AsCacheable, B = AsCacheable, C = AsCacheable> {
  _target: (A, B, C),
}

impl<A, B, C, K, V, H> ArchiveWith<(K, V, H)> for AsTuple3<A, B, C>
where
  A: ArchiveWith<K>,
  B: ArchiveWith<V>,
  C: ArchiveWith<H>,
{
  type Archived = Tuple3<A::Archived, B::Archived, C::Archived>;
  type Resolver = Tuple3<A::Resolver, B::Resolver, C::Resolver>;

  #[inline]
  unsafe fn resolve_with(
    field: &(K, V, H),
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let (fp, fo) = out_field!(out.a);
    A::resolve_with(&field.0, pos + fp, resolver.a, fo);

    let (fp, fo) = out_field!(out.b);
    B::resolve_with(&field.1, pos + fp, resolver.b, fo);

    let (fp, fo) = out_field!(out.c);
    C::resolve_with(&field.2, pos + fp, resolver.c, fo);
  }
}

impl<A, B, C, K, V, H, S: Fallible + ?Sized> SerializeWith<(K, V, H), S> for AsTuple3<A, B, C>
where
  A: SerializeWith<K, S>,
  B: SerializeWith<V, S>,
  C: SerializeWith<H, S>,
{
  #[inline]
  fn serialize_with(field: &(K, V, H), serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok(Tuple3 {
      a: A::serialize_with(&field.0, serializer)?,
      b: B::serialize_with(&field.1, serializer)?,
      c: C::serialize_with(&field.2, serializer)?,
    })
  }
}

impl<A, B, C, T> CheckBytes<T> for Tuple3<A, B, C>
where
  A: CheckBytes<T>,
  B: CheckBytes<T>,
  C: CheckBytes<T>,
  T: ArchiveContext + ?Sized,
{
  type Error = DeserializeError;

  #[inline]
  unsafe fn check_bytes<'a>(value: *const Self, context: &mut T) -> Result<&'a Self, Self::Error> {
    A::check_bytes(core::ptr::addr_of!((*value).a), context)
      .map_err(|_| DeserializeError::CheckBytesError)?;
    B::check_bytes(core::ptr::addr_of!((*value).b), context)
      .map_err(|_| DeserializeError::CheckBytesError)?;
    C::check_bytes(core::ptr::addr_of!((*value).c), context)
      .map_err(|_| DeserializeError::CheckBytesError)?;
    Ok(&*value)
  }
}

impl<A, B, C, K, V, H, D>
  DeserializeWith<Tuple3<A::Archived, B::Archived, C::Archived>, (K, V, H), D> for AsTuple3<A, B, C>
where
  A: ArchiveWith<K> + DeserializeWith<A::Archived, K, D>,
  B: ArchiveWith<V> + DeserializeWith<B::Archived, V, D>,
  C: ArchiveWith<H> + DeserializeWith<C::Archived, H, D>,
  D: ?Sized + Fallible,
{
  fn deserialize_with(
    field: &Tuple3<A::Archived, B::Archived, C::Archived>,
    deserializer: &mut D,
  ) -> Result<(K, V, H), D::Error> {
    Ok((
      A::deserialize_with(&field.a, deserializer)?,
      B::deserialize_with(&field.b, deserializer)?,
      C::deserialize_with(&field.c, deserializer)?,
    ))
  }
}
