use rspack_cacheable::{
  Error, Result,
  rkyv::{
    Place, SerializeUnsized,
    de::{ErasedPtr, Pooling, PoolingState},
    rancor::Fallible,
    rc::{ArchivedRc, Flavor, RcResolver},
    ser::{Allocator, Sharing, Writer, sharing::SharingState},
    vec::ArchivedVec,
    with::{ArchiveWith, AsVec, DeserializeWith, SerializeWith, With},
  },
};

use crate::ProjectPath;

/// Context-free `rkyv` serializer/deserializer adapter for [`ProjectPath`].
pub struct AsProjectPath;

type ArchivedProjectPath = ArchivedRc<ArchivedVec<u8>, AsProjectPath>;

impl Flavor for AsProjectPath {
  const ALLOW_CYCLES: bool = false;
}

impl ArchiveWith<ProjectPath> for AsProjectPath {
  type Archived = ArchivedProjectPath;
  type Resolver = RcResolver;

  #[inline]
  fn resolve_with(_field: &ProjectPath, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedRc::resolve_from_ref(With::<&[u8], AsVec>::cast(&&[][..]), resolver, out);
  }
}

impl<S> SerializeWith<ProjectPath, S> for AsProjectPath
where
  S: Fallible<Error = Error> + Writer + Allocator + Sharing + ?Sized,
{
  #[inline]
  fn serialize_with(field: &ProjectPath, serializer: &mut S) -> Result<Self::Resolver> {
    let address = field.header_addr();
    let pos = match serializer.start_sharing(address) {
      SharingState::Started => {
        let items = field.raw_items();
        let tag = field.wire_tag();
        let total_len = 1 + items.len();
        let pos = if total_len <= 512 {
          let mut buf = [0u8; 512];
          buf[0] = tag;
          buf[1..total_len].copy_from_slice(items);
          With::<_, AsVec>::cast(&&buf[..total_len]).serialize_unsized(serializer)?
        } else {
          let mut buf = Vec::with_capacity(total_len);
          buf.push(tag);
          buf.extend_from_slice(items);
          With::<_, AsVec>::cast(&buf.as_slice()).serialize_unsized(serializer)?
        };
        serializer.finish_sharing(address, pos)?;
        pos
      }
      SharingState::Finished(pos) => pos,
      SharingState::Pending => {
        return Err(Error::MessageError("cyclic shared project path"));
      }
    };
    Ok(RcResolver::from_pos(pos))
  }
}

impl<D> DeserializeWith<ArchivedProjectPath, ProjectPath, D> for AsProjectPath
where
  D: Fallible<Error = Error> + Pooling + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedProjectPath, deserializer: &mut D) -> Result<ProjectPath> {
    unsafe fn drop_pooled_project_path(ptr: ErasedPtr) {
      // SAFETY: `ptr` was allocated via `Box::into_raw(Box::new(path))` in `Started` below.
      unsafe {
        drop(Box::from_raw(ptr.data_address().cast::<ProjectPath>()));
      }
    }

    // `field.get()` points into the archive buffer and is guaranteed non-zero, so it never collides
    // with `ContextGuard`'s sentinel `CONTEXT_ADDR = 0`.
    let address = std::ptr::from_ref(field.get()).addr();
    match deserializer.start_pooling(address) {
      PoolingState::Started => {
        let path = ProjectPath::from_wire_bytes(field.as_slice())?;
        let raw = Box::into_raw(Box::new(path.clone()));
        // SAFETY: `raw` is exclusively owned and `drop_pooled_project_path` drops it when the pool drops.
        unsafe {
          deserializer.finish_pooling(address, ErasedPtr::new(raw), drop_pooled_project_path)?;
        }
        Ok(path)
      }
      PoolingState::Pending => Err(Error::MessageError("cyclic shared project path")),
      PoolingState::Finished(ptr) => {
        // SAFETY: `Finished` holds the `Box<ProjectPath>` registered under `address`.
        let path = unsafe { &*ptr.data_address().cast::<ProjectPath>() };
        Ok(path.clone())
      }
    }
  }
}
