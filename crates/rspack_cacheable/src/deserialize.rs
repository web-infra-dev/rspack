use rkyv::{
  Archive, Deserialize, access,
  api::{deserialize_using, high::HighValidator},
  bytecheck::CheckBytes,
  de::Pool,
  rancor::Strategy,
  util::AlignedVec,
};

use crate::{
  context::{CacheableContext, ContextGuard},
  error::{Error, Result},
};

pub type Validator<'a> = HighValidator<'a, Error>;
pub type Deserializer = Strategy<Pool, Error>;

/// Transform bytes to struct
///
/// This function implementation refers to rkyv::from_bytes and
/// add custom error and context support
pub fn from_bytes<T, C: CacheableContext>(bytes: &[u8], context: &C) -> Result<T>
where
  T: Archive,
  T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
{
  let guard = ContextGuard::new(context);
  let mut deserializer = Pool::default();
  guard.add_to_pooling(&mut deserializer)?;
  let mut aligned_vec = AlignedVec::<16>::new();
  let aligned_bytes = if (bytes.as_ptr() as usize).is_multiple_of(16) {
    bytes
  } else {
    // Rkyv validates archived pointers against their alignment. Preserve the
    // existing realignment behavior for subslices and other unaligned inputs.
    aligned_vec.extend_from_slice(bytes);
    aligned_vec.as_slice()
  };
  deserialize_using(
    access::<T::Archived, Error>(aligned_bytes)?,
    &mut deserializer,
  )
}
