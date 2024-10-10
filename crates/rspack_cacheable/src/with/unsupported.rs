use rkyv::{
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use crate::{DeserializeError, Deserializer, SerializeError, Serializer};

pub struct Unsupported;

impl<F> ArchiveWith<F> for Unsupported {
  type Archived = ();
  type Resolver = ();

  fn resolve_with(_: &F, _: Self::Resolver, _: Place<Self::Archived>) {}
}

impl<'a, F> SerializeWith<F, Serializer<'a>> for Unsupported {
  fn serialize_with(_: &F, _: &mut Serializer) -> Result<(), SerializeError> {
    Err(SerializeError::SerializeFailed("unsupported field"))
  }
}

impl<F> DeserializeWith<(), F, Deserializer> for Unsupported {
  fn deserialize_with(_: &(), _: &mut Deserializer) -> Result<F, DeserializeError> {
    Err(DeserializeError::DeserializeFailed("unsupported field"))
  }
}
