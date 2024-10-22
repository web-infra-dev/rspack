use core::marker::PhantomData;
use std::{
  collections::HashMap,
  hash::{Hash, Hasher},
};

use inventory;
use rkyv::{
  bytecheck::{CheckBytes, StructCheckContext},
  ptr_meta::{DynMetadata, Pointee},
  rancor::{Fallible, Trace},
  traits::NoUndef,
  Archived, Portable, SerializeUnsized,
};

pub mod validation;
use crate::{DeserializeError, Deserializer, SerializeError, Serializer};

/// A trait object that can be archived.
pub trait SerializeDyn {
  /// Writes the value to the serializer and returns the position it was written to.
  fn serialize_dyn(&self, serializer: &mut Serializer) -> Result<usize, SerializeError>;
}

impl<T> SerializeDyn for T
where
  T: for<'a> SerializeUnsized<Serializer<'a>>,
{
  fn serialize_dyn(&self, serializer: &mut Serializer) -> Result<usize, SerializeError> {
    self.serialize_unsized(serializer)
  }
}

/// A trait object that can be deserialized.
///
/// See [`SerializeDyn`] for more information.
pub trait DeserializeDyn<T: Pointee + ?Sized> {
  /// Deserializes this value into the given out pointer.
  fn deserialize_dyn(
    &self,
    deserializer: &mut Deserializer,
    out: *mut T,
  ) -> Result<(), DeserializeError>;

  /// Returns the pointer metadata for the deserialized form of this type.
  fn deserialized_pointer_metadata(&self) -> DynMetadata<T>;
}

/// The archived version of `DynMetadata`.
pub struct ArchivedDynMetadata<T: ?Sized> {
  dyn_id: Archived<u64>,
  phantom: PhantomData<T>,
}

impl<T: ?Sized> Default for ArchivedDynMetadata<T> {
  fn default() -> Self {
    Self {
      dyn_id: Archived::<u64>::from_native(0),
      phantom: PhantomData::default(),
    }
  }
}
impl<T: ?Sized> Hash for ArchivedDynMetadata<T> {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    Hash::hash(&self.dyn_id, state);
  }
}
impl<T: ?Sized> PartialEq for ArchivedDynMetadata<T> {
  #[inline]
  fn eq(&self, other: &ArchivedDynMetadata<T>) -> bool {
    self.dyn_id == other.dyn_id
  }
}
impl<T: ?Sized> Eq for ArchivedDynMetadata<T> {}
impl<T: ?Sized> PartialOrd for ArchivedDynMetadata<T> {
  #[inline]
  fn partial_cmp(&self, other: &ArchivedDynMetadata<T>) -> Option<::core::cmp::Ordering> {
    Some(self.dyn_id.cmp(&other.dyn_id))
  }
}
impl<T: ?Sized> Ord for ArchivedDynMetadata<T> {
  #[inline]
  fn cmp(&self, other: &ArchivedDynMetadata<T>) -> ::core::cmp::Ordering {
    self.dyn_id.cmp(&other.dyn_id)
  }
}
impl<T: ?Sized> Clone for ArchivedDynMetadata<T> {
  fn clone(&self) -> ArchivedDynMetadata<T> {
    *self
  }
}
impl<T: ?Sized> Copy for ArchivedDynMetadata<T> {}
impl<T: ?Sized> Unpin for ArchivedDynMetadata<T> {}
unsafe impl<T: ?Sized> Sync for ArchivedDynMetadata<T> {}
unsafe impl<T: ?Sized> Send for ArchivedDynMetadata<T> {}
unsafe impl<T: ?Sized> NoUndef for ArchivedDynMetadata<T> {}
unsafe impl<T: ?Sized> Portable for ArchivedDynMetadata<T> {}
unsafe impl<T: ?Sized, C> CheckBytes<C> for ArchivedDynMetadata<T>
where
  C: Fallible + ?Sized,
  C::Error: Trace,
  Archived<u64>: CheckBytes<C>,
  PhantomData<T>: CheckBytes<C>,
{
  unsafe fn check_bytes(
    value: *const Self,
    context: &mut C,
  ) -> ::core::result::Result<(), C::Error> {
    Archived::<u64>::check_bytes(&raw const (*value).dyn_id, context).map_err(|e| {
      C::Error::trace(
        e,
        StructCheckContext {
          struct_name: "ArchivedDynMetadata",
          field_name: "dyn_id",
        },
      )
    })?;
    PhantomData::<T>::check_bytes(&raw const (*value).phantom, context).map_err(|e| {
      C::Error::trace(
        e,
        StructCheckContext {
          struct_name: "ArchivedDynMetadata",
          field_name: "phantom",
        },
      )
    })?;
    Ok(())
  }
}

impl<T: ?Sized> ArchivedDynMetadata<T> {
  pub fn new(dyn_id: u64) -> Self {
    Self {
      dyn_id: Archived::<u64>::from_native(dyn_id),
      phantom: PhantomData,
    }
  }

  /// Returns the pointer metadata for the trait object this metadata refers to.
  pub fn lookup_metadata(&self) -> DynMetadata<T> {
    unsafe {
      std::mem::transmute(
        *DYN_REGISTRY
          .get(&self.dyn_id.to_native())
          .expect("attempted to get vtable for an unregistered impl"),
      )
    }
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
