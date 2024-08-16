use core::{marker::PhantomData, ptr};
use std::collections::HashMap;

use rkyv::{from_archived, validation::validators::DefaultValidator, Archived, CheckBytes};

use super::{ArchivedDynMetadata, DYN_REGISTRY};
use crate::DeserializeError;

type CheckBytesDyn =
  unsafe fn(*const u8, &mut DefaultValidator<'_>) -> Result<(), DeserializeError>;

pub unsafe fn default_check_bytes_dyn<T>(
  bytes: *const u8,
  context: &mut DefaultValidator<'_>,
) -> Result<(), DeserializeError>
where
  T: for<'a> CheckBytes<DefaultValidator<'a>>,
{
  match T::check_bytes(bytes.cast(), context) {
    Ok(_) => Ok(()),
    Err(_) => Err(DeserializeError::CheckBytesError),
  }
}

impl<T: ?Sized, C: ?Sized> CheckBytes<C> for ArchivedDynMetadata<T> {
  type Error = DeserializeError;

  unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
    let Ok(dyn_id) = Archived::<u64>::check_bytes(ptr::addr_of!((*value).dyn_id), context) else {
      return Err(DeserializeError::CheckBytesError);
    };
    if PhantomData::<T>::check_bytes(ptr::addr_of!((*value).phantom), context).is_err() {
      return Err(DeserializeError::CheckBytesError);
    }

    let cached_vtable_ptr = ptr::addr_of!((*value).cached_vtable);
    let Ok(cached_vtable) = Archived::<u64>::check_bytes(cached_vtable_ptr, context) else {
      return Err(DeserializeError::CheckBytesError);
    };

    if let Some(impl_data) = DYN_REGISTRY.get(&from_archived!(*dyn_id)) {
      let cached_vtable = from_archived!(*cached_vtable);
      if cached_vtable == 0 || &(cached_vtable as usize) == impl_data {
        return Ok(&*value);
      }
    }
    Err(DeserializeError::CheckBytesError)
  }
}

pub struct CheckBytesEntry {
  vtable: usize,
  check_bytes_dyn: CheckBytesDyn,
}

impl CheckBytesEntry {
  #[doc(hidden)]
  pub fn new(vtable: usize, check_bytes_dyn: CheckBytesDyn) -> Self {
    Self {
      vtable,
      check_bytes_dyn,
    }
  }
}

inventory::collect!(CheckBytesEntry);

pub static CHECK_BYTES_REGISTRY: std::sync::LazyLock<HashMap<usize, CheckBytesDyn>> =
  std::sync::LazyLock::new(|| {
    let mut result = HashMap::default();
    for entry in inventory::iter::<CheckBytesEntry> {
      let old_value = result.insert(entry.vtable, entry.check_bytes_dyn);
      if old_value.is_some() {
        panic!("vtable conflict, a trait implementation was likely added twice (but it's possible there was a hash collision)")
      }
    }
    result
  });
