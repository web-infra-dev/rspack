use std::{path::PathBuf, ptr};

use rspack_cacheable::{
  ContextGuard, Error, Result,
  rkyv::{
    Place, SerializeUnsized,
    de::Pooling,
    rancor::Fallible,
    rc::{ArchivedRc, Flavor, RcResolver},
    ser::{Sharing, Writer, sharing::SharingState},
    with::{ArchiveWith, DeserializeWith, SerializeWith, With},
  },
  utils::PortablePath,
  with::{Custom, CustomConverter},
};

use crate::InternedPath;

/// Shares interned allocations within an archive while preserving portable path conversion.
pub struct AsInternedPath;

impl Flavor for AsInternedPath {
  const ALLOW_CYCLES: bool = false;
}

impl ArchiveWith<InternedPath> for AsInternedPath {
  type Archived = ArchivedRc<<Custom as ArchiveWith<InternedPath>>::Archived, Self>;
  type Resolver = RcResolver;

  fn resolve_with(field: &InternedPath, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedRc::resolve_from_ref(With::<_, Custom>::cast(field), resolver, out);
  }
}

impl<S> SerializeWith<InternedPath, S> for AsInternedPath
where
  S: Fallible<Error = Error> + Writer + Sharing + ?Sized,
{
  fn serialize_with(field: &InternedPath, serializer: &mut S) -> Result<Self::Resolver> {
    // Key sharing by the interned allocation, not the handle or the temporary PortablePath.
    // The header has a stable, unique address even for an empty path.
    let address = ptr::from_ref(field.0.header()).addr();
    let pos = match serializer.start_sharing(address) {
      SharingState::Started => {
        let pos = With::<_, Custom>::cast(field).serialize_unsized(serializer)?;
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

impl<D> DeserializeWith<<Self as ArchiveWith<InternedPath>>::Archived, InternedPath, D>
  for AsInternedPath
where
  D: Fallible<Error = Error> + Pooling + ?Sized,
{
  fn deserialize_with(
    field: &<Self as ArchiveWith<InternedPath>>::Archived,
    deserializer: &mut D,
  ) -> Result<InternedPath> {
    Custom::deserialize_with(field.get(), deserializer)
  }
}

impl CustomConverter for InternedPath {
  type Target = PortablePath;

  fn serialize(&self, guard: &ContextGuard) -> Result<Self::Target> {
    Ok(PortablePath::new(self.as_path(), guard.project_root()))
  }

  fn deserialize(data: Self::Target, guard: &ContextGuard) -> Result<Self> {
    Ok(Self::from(PathBuf::from(
      data.into_path_string(guard.project_root()),
    )))
  }
}
