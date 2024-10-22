use std::collections::HashMap;

use rkyv::bytecheck::CheckBytes;

use crate::{DeserializeError, Validator};

type CheckBytesDyn = unsafe fn(*const u8, &mut Validator<'_>) -> Result<(), DeserializeError>;

/// # Safety
///
/// Run T::check_bytes
pub unsafe fn default_check_bytes_dyn<T>(
  bytes: *const u8,
  context: &mut Validator<'_>,
) -> Result<(), DeserializeError>
where
  T: for<'a> CheckBytes<Validator<'a>>,
{
  T::check_bytes(bytes.cast(), context)
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
