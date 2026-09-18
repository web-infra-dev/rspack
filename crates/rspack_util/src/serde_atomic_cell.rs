use crossbeam_utils::atomic::AtomicCell;
use serde::{Serialize, Serializer};

/// Serialize a copy of the value stored in an atomic cell.
pub fn serialize<T: Copy + Serialize, S: Serializer>(
  value: &AtomicCell<T>,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  value.load().serialize(serializer)
}

/// Skip an absent atomic value with Serde's `skip_serializing_if` attribute.
/// The value must remain unchanged between this check and serialization.
pub fn is_none<T: Copy>(value: &AtomicCell<Option<T>>) -> bool {
  value.load().is_none()
}
