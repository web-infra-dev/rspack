use std::ptr;

use rspack_cacheable::{
  ContextGuard, Error, Result,
  rkyv::{
    Archive, Deserialize, Place, Serialize, SerializeUnsized,
    de::Pooling,
    rancor::Fallible,
    rc::{ArchivedRc, Flavor, RcResolver},
    ser::{Allocator, Sharing, Writer, sharing::SharingState},
    with::{ArchiveWith, AsVec, DeserializeWith, SerializeWith},
  },
  utils::PortablePath,
};

use crate::InternedPath;

#[derive(Archive, Serialize)]
#[rkyv(crate = rspack_cacheable::rkyv, derive(Hash, PartialEq, Eq))]
#[rkyv(archive_bounds(
  <AsVec as ArchiveWith<&'a [u8]>>::Archived: std::hash::Hash + Eq
))]
pub enum InternedPathData<'a> {
  Native(#[rkyv(with = AsVec)] &'a [u8]),
  Portable(PortablePath),
}

/// Shares path data within an archive, relative to the project root in portable mode.
pub struct AsInternedPath;

// AsVec stores the bytes in the archive without retaining the source borrow.
type ArchivedInternedPath = ArchivedRc<ArchivedInternedPathData<'static>, AsInternedPath>;

impl Flavor for AsInternedPath {
  const ALLOW_CYCLES: bool = false;
}

impl ArchiveWith<InternedPath> for AsInternedPath {
  type Archived = ArchivedInternedPath;
  type Resolver = RcResolver;

  #[inline]
  fn resolve_with(_field: &InternedPath, resolver: Self::Resolver, out: Place<Self::Archived>) {
    // The payload is sized, so only the resolver's position is needed.
    ArchivedRc::resolve_from_ref(&InternedPathData::Native(&[]), resolver, out);
  }
}

impl<S> SerializeWith<InternedPath, S> for AsInternedPath
where
  S: Fallible<Error = Error> + Writer + Allocator + Sharing + ?Sized,
{
  #[inline]
  fn serialize_with(field: &InternedPath, serializer: &mut S) -> Result<Self::Resolver> {
    // The interned header provides a stable sharing key, including for empty paths.
    let address = ptr::from_ref(field.0.header()).addr();
    let pos = match serializer.start_sharing(address) {
      SharingState::Started => {
        let guard = ContextGuard::sharing_guard(serializer)?;
        let data = if let Some(root) = guard.project_root() {
          InternedPathData::Portable(PortablePath::new(field.as_path(), Some(root)))
        } else {
          // Preserve native bytes, including non-UTF-8 paths, outside portable mode.
          InternedPathData::Native(field.0.items())
        };
        let pos = data.serialize_unsized(serializer)?;
        serializer.finish_sharing(address, pos)?;
        pos
      }
      SharingState::Finished(pos) => pos,
      SharingState::Pending => {
        return Err(Error::MessageError("cyclic shared interned path"));
      }
    };
    Ok(RcResolver::from_pos(pos))
  }
}

impl<D> DeserializeWith<ArchivedInternedPath, InternedPath, D> for AsInternedPath
where
  D: Fallible<Error = Error> + Pooling<Error> + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedInternedPath, deserializer: &mut D) -> Result<InternedPath> {
    match field.get() {
      ArchivedInternedPathData::Native(bytes) => Ok(InternedPath::from_bytes(bytes.as_slice())),
      ArchivedInternedPathData::Portable(path) => {
        let path: PortablePath = path.deserialize(deserializer)?;
        let guard = ContextGuard::pooling_guard(deserializer)?;
        Ok(InternedPath::from(
          path.into_path_string(guard.project_root()).as_str(),
        ))
      }
    }
  }
}
