use rkyv::with::{ArchiveWith, DeserializeWith, SerializeWith};

use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

pub struct Unsupported;

impl<F> ArchiveWith<F> for Unsupported {
  type Archived = ();
  type Resolver = ();

  unsafe fn resolve_with(_: &F, _: usize, _: Self::Resolver, _: *mut Self::Archived) {}
}

impl<F> SerializeWith<F, CacheableSerializer> for Unsupported {
  fn serialize_with(_: &F, _: &mut CacheableSerializer) -> Result<(), SerializeError> {
    Err(SerializeError::SerializeFailed("unsupported field"))
  }
}

impl<F> DeserializeWith<(), F, CacheableDeserializer> for Unsupported {
  fn deserialize_with(_: &(), _: &mut CacheableDeserializer) -> Result<F, DeserializeError> {
    Err(DeserializeError::DeserializeFailed("unsupported field"))
  }
}
