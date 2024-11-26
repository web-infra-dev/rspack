use rkyv::{
  rancor::Fallible,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use crate::{DeserializeError, SerializeError};

pub struct Unsupported;

impl<F> ArchiveWith<F> for Unsupported {
  type Archived = ();
  type Resolver = ();

  fn resolve_with(_: &F, _: Self::Resolver, _: Place<Self::Archived>) {}
}

impl<F, S> SerializeWith<F, S> for Unsupported
where
  S: Fallible<Error = SerializeError> + ?Sized,
{
  fn serialize_with(_: &F, _: &mut S) -> Result<(), SerializeError> {
    Err(SerializeError::UnsupportedField)
  }
}

impl<F, D> DeserializeWith<(), F, D> for Unsupported
where
  D: Fallible<Error = DeserializeError> + ?Sized,
{
  fn deserialize_with(_: &(), _: &mut D) -> Result<F, DeserializeError> {
    Err(DeserializeError::UnsupportedField)
  }
}
