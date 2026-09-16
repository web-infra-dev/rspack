use std::sync::Arc;

use rkyv::{
  Archive, ArchiveUnsized, Place, Serialize,
  de::{ErasedPtr, Pooling, PoolingState},
  ptr_meta::{self, DynMetadata, Pointee},
  rancor::Fallible,
  rc::{ArcFlavor, ArchivedRc},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Deserializer, Error, Result, r#dyn::DeserializeArcDyn};

/// Restores shared trait objects directly into an Arc allocation while preserving
/// the archive format and reference sharing of Arc.
pub struct AsArc;

impl<T: ?Sized> ArchiveWith<Arc<T>> for AsArc
where
  Arc<T>: Archive,
{
  type Archived = <Arc<T> as Archive>::Archived;
  type Resolver = <Arc<T> as Archive>::Resolver;

  fn resolve_with(field: &Arc<T>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    field.resolve(resolver, out);
  }
}

impl<T: ?Sized, S: Fallible + ?Sized> SerializeWith<Arc<T>, S> for AsArc
where
  Arc<T>: Serialize<S>,
{
  fn serialize_with(field: &Arc<T>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    field.serialize(serializer)
  }
}

impl<T> DeserializeWith<ArchivedRc<T::Archived, ArcFlavor>, Arc<T>, Deserializer> for AsArc
where
  T: ArchiveUnsized + Pointee<Metadata = DynMetadata<T>> + ?Sized,
  T::Archived: DeserializeArcDyn<T>,
{
  fn deserialize_with(
    field: &ArchivedRc<T::Archived, ArcFlavor>,
    deserializer: &mut Deserializer,
  ) -> Result<Arc<T>> {
    let archived = field.get();
    let address = archived as *const T::Archived as *const () as usize;
    match deserializer.start_pooling(address) {
      PoolingState::Started => {
        let value = archived.deserialize_arc(deserializer)?;
        let pooled = Arc::into_raw(Arc::clone(&value));
        // SAFETY: `pooled` owns one strong reference, which the pool releases
        // through `drop_arc`. The concrete Pool takes ownership only on success.
        if let Err(error) = unsafe {
          deserializer.finish_pooling(address, ErasedPtr::new(pooled.cast_mut()), drop_arc::<T>)
        } {
          // SAFETY: registration failed, so the pool does not own this reference.
          unsafe { drop(Arc::from_raw(pooled)) };
          return Err(error);
        }
        Ok(value)
      }
      PoolingState::Pending => Err(Error::MessageError("cyclic shared trait object")),
      PoolingState::Finished(pooled) => {
        // Reconstruct metadata from this archived trait object: another view of
        // the same allocation may have registered different pointer metadata.
        let ptr = ptr_meta::from_raw_parts::<T>(
          pooled.data_address(),
          archived.deserialized_pointer_metadata(),
        );
        // SAFETY: the pool keeps a strong reference to this restored allocation.
        // Increment it before returning another owning Arc.
        unsafe {
          Arc::increment_strong_count(ptr);
          Ok(Arc::from_raw(ptr))
        }
      }
    }
  }
}

unsafe fn drop_arc<T: Pointee<Metadata = DynMetadata<T>> + ?Sized>(ptr: ErasedPtr) {
  // SAFETY: finish_pooling pairs this callback with a pointer from Arc<T>::into_raw.
  unsafe { drop(Arc::from_raw(ptr.downcast_unchecked::<T>())) };
}
