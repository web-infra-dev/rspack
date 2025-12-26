use rkyv::{
  Place,
  rancor::Fallible,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result};

pub struct Unsupported;

impl<F> ArchiveWith<F> for Unsupported {
  type Archived = ();
  type Resolver = ();

  fn resolve_with(_: &F, _: Self::Resolver, _: Place<Self::Archived>) {}
}

impl<F, S> SerializeWith<F, S> for Unsupported
where
  S: Fallible<Error = Error> + ?Sized,
{
  fn serialize_with(_: &F, _: &mut S) -> Result<()> {
    Err(Error::UnsupportedField)
  }
}

impl<F, D> DeserializeWith<(), F, D> for Unsupported
where
  D: Fallible<Error = Error> + ?Sized,
{
  fn deserialize_with(_: &(), _: &mut D) -> Result<F> {
    Err(Error::UnsupportedField)
  }
}
