use std::any::Any;

use rkyv::{
  Archive, Deserialize, access,
  api::{deserialize_using, high::HighValidator},
  bytecheck::CheckBytes,
  de::Pool,
  rancor::Strategy,
  util::AlignedVec,
};

use crate::{
  context::ContextGuard,
  error::{Error, Result},
};

pub type Validator<'a> = HighValidator<'a, Error>;
pub type Deserializer = Strategy<Pool, Error>;

/// Transform bytes to struct
///
/// This function implementation refers to rkyv::from_bytes and
/// add custom error and context support
pub fn from_bytes<T, C: Any>(bytes: &[u8], context: &C) -> Result<T>
where
  T: Archive,
  T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
{
  let guard = ContextGuard::new(context);
  let mut deserializer = Pool::default();
  guard.add_to_pooling(&mut deserializer)?;
  // The `bytes` ptr address in miri will throw UnalignedPointer error in rkyv.
  // AlignedVec will force aligned the ptr address.
  // Refer code: https://github.com/rkyv/rkyv/blob/dabbc1fcf5052f141403b84493bddb74c44f9ba9/rkyv/src/validation/archive/validator.rs#L135
  let mut aligned_vec = AlignedVec::<16>::new();
  aligned_vec.extend_from_slice(bytes);
  deserialize_using(
    access::<T::Archived, Error>(&aligned_vec)?,
    &mut deserializer,
  )
}
