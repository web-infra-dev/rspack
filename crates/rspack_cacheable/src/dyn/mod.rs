use core::{alloc::Layout, marker::PhantomData, ptr};
use std::collections::HashMap;

use inventory;
use ptr_meta::{DynMetadata, Pointee};
use rkyv::{from_archived, ser::Serializer, to_archived, Archived, Serialize};

pub mod validation;

use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

/// A trait object that can be archived.
pub trait SerializeDyn {
  /// Writes the value to the serializer and returns the position it was written to.
  fn serialize_dyn(&self, serializer: &mut CacheableSerializer) -> Result<usize, SerializeError>;
}

impl<T: Serialize<CacheableSerializer>> SerializeDyn for T {
  fn serialize_dyn(&self, serializer: &mut CacheableSerializer) -> Result<usize, SerializeError> {
    serializer.serialize_value(self)
  }
}

/// A trait object forked from rkyv_dyn::DeserializeDyn
///
/// This trait will override some internal methods params.
/// 1. deserializer from `&mut dyn DynDeserializer` to `CacheableDeserializer`
/// 2. return Error from `DynError` to `DeserializeError`
pub trait DeserializeDyn<T: Pointee + ?Sized> {
  /// Deserializes the given value as a trait object.
  unsafe fn deserialize_dyn(
    &self,
    deserializer: &mut CacheableDeserializer,
    alloc: &mut dyn FnMut(Layout) -> *mut u8,
  ) -> Result<*mut (), DeserializeError>;

  /// Returns the metadata for the deserialized version of this value.
  fn deserialize_dyn_metadata(
    &self,
    deserializer: &mut CacheableDeserializer,
  ) -> Result<T::Metadata, DeserializeError>;
}

/// The archived version of `DynMetadata`.
pub struct ArchivedDynMetadata<T: ?Sized> {
  dyn_id: Archived<u64>,
  cached_vtable: Archived<u64>,
  phantom: PhantomData<T>,
}

impl<T: ?Sized> ArchivedDynMetadata<T> {
  /// Creates a new `ArchivedDynMetadata` for the given type.
  ///
  /// # Safety
  ///
  /// `out` must point to a valid location for an `ArchivedDynMetadata<T>`.
  pub unsafe fn emplace(dyn_id: u64, out: *mut Self) {
    ptr::addr_of_mut!((*out).dyn_id).write(to_archived!(dyn_id));
    ptr::addr_of_mut!((*out).cached_vtable).write(to_archived!(0u64));
  }

  pub fn vtable(&self) -> usize {
    *DYN_REGISTRY
      .get(from_archived!(&self.dyn_id))
      .expect("attempted to get vtable for an unregistered impl")
  }

  /// Gets the `DynMetadata` associated with this `ArchivedDynMetadata`.
  pub fn pointer_metadata(&self) -> DynMetadata<T> {
    unsafe { core::mem::transmute(self.vtable()) }
  }
}

pub struct DynEntry {
  dyn_id: u64,
  vtable: usize,
}

impl DynEntry {
  pub fn new(dyn_id: u64, vtable: usize) -> Self {
    Self { dyn_id, vtable }
  }
}

inventory::collect!(DynEntry);

static DYN_REGISTRY: std::sync::LazyLock<HashMap<u64, usize>> = std::sync::LazyLock::new(|| {
  let mut result = HashMap::default();
  for entry in inventory::iter::<DynEntry> {
    let old_value = result.insert(entry.dyn_id, entry.vtable);
    if old_value.is_some() {
      panic!("cacheable_dyn init global REGISTRY error, duplicate implementation.")
    }
  }
  result
});
