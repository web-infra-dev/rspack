use std::ptr;

use rspack_cacheable::{
  Error, Result,
  rkyv::{
    Place, SerializeUnsized,
    rancor::Fallible,
    rc::{ArchivedRc, Flavor, RcResolver},
    ser::{Allocator, Sharing, Writer, sharing::SharingState},
    vec::ArchivedVec,
    with::{ArchiveWith, AsVec, DeserializeWith, SerializeWith, With},
  },
};

use crate::InternedPath;

/// Shares native path bytes within an archive.
pub struct AsInternedPath;

type ArchivedInternedPath = ArchivedRc<ArchivedVec<u8>, AsInternedPath>;

impl Flavor for AsInternedPath {
  const ALLOW_CYCLES: bool = false;
}

impl ArchiveWith<InternedPath> for AsInternedPath {
  type Archived = ArchivedInternedPath;
  type Resolver = RcResolver;

  #[inline]
  fn resolve_with(_field: &InternedPath, resolver: Self::Resolver, out: Place<Self::Archived>) {
    // The payload is sized, so only the resolver's position is needed.
    ArchivedRc::resolve_from_ref(With::<&[u8], AsVec>::cast(&&[][..]), resolver, out);
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
        // InternedPath stores the complete slice from OsStr::as_encoded_bytes().
        let pos = With::<_, AsVec>::cast(&field.0.items()).serialize_unsized(serializer)?;
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
  D: Fallible<Error = Error> + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedInternedPath, _deserializer: &mut D) -> Result<InternedPath> {
    Ok(InternedPath::from_bytes(field.as_slice()))
  }
}
