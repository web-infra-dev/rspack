use rkyv::{
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

pub struct AsBytes;

pub trait AsBytesConverter {
  // todo change return to Result<Cow<Vec<u8>>, SerializeError>
  fn to_bytes(&self) -> Result<Vec<u8>, SerializeError>;
  fn from_bytes(s: &[u8]) -> Result<Self, DeserializeError>
  where
    Self: Sized;
}

pub struct AsBytesResolver {
  inner: VecResolver,
  len: usize,
}

impl<T> ArchiveWith<T> for AsBytes {
  type Archived = ArchivedVec<u8>;
  type Resolver = AsBytesResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedVec::resolve_from_len(resolver.len, pos, resolver.inner, out)
  }
}

impl<T> SerializeWith<T, CacheableSerializer> for AsBytes
where
  T: AsBytesConverter,
{
  #[inline]
  fn serialize_with(
    field: &T,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let bytes = &field.to_bytes()?;
    Ok(AsBytesResolver {
      inner: ArchivedVec::serialize_from_slice(bytes, serializer)?,
      len: bytes.len(),
    })
  }
}

impl<T> DeserializeWith<ArchivedVec<u8>, T, CacheableDeserializer> for AsBytes
where
  T: AsBytesConverter,
{
  #[inline]
  fn deserialize_with(
    field: &ArchivedVec<u8>,
    _de: &mut CacheableDeserializer,
  ) -> Result<T, DeserializeError> {
    AsBytesConverter::from_bytes(field)
  }
}
