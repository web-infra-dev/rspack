use camino::Utf8PathBuf;
use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  de::Pooling,
  rancor::Fallible,
  ser::{Sharing, Writer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{ContextGuard, Error, utils::PortablePath};

pub struct PathResolver {
  inner: Resolver<PortablePath>,
  path: PortablePath,
}

impl ArchiveWith<Utf8PathBuf> for AsPreset {
  type Archived = Archived<PortablePath>;
  type Resolver = PathResolver;

  #[inline]
  fn resolve_with(_field: &Utf8PathBuf, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let PathResolver { inner, path } = resolver;
    Archive::resolve(&path, inner, out);
  }
}

impl<S> SerializeWith<Utf8PathBuf, S> for AsPreset
where
  S: ?Sized + Fallible<Error = Error> + Writer + Sharing<Error>,
{
  #[inline]
  fn serialize_with(field: &Utf8PathBuf, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let guard = ContextGuard::sharing_guard(serializer)?;
    let portable_path = PortablePath::new(field.as_ref(), guard.project_root());
    Ok(PathResolver {
      inner: Serialize::serialize(&portable_path, serializer)?,
      path: portable_path,
    })
  }
}

impl<D> DeserializeWith<Archived<PortablePath>, Utf8PathBuf, D> for AsPreset
where
  D: ?Sized + Fallible<Error = Error> + Pooling<Error>,
{
  #[inline]
  fn deserialize_with(field: &Archived<PortablePath>, de: &mut D) -> Result<Utf8PathBuf, D::Error> {
    let portable_path: PortablePath = Deserialize::deserialize(field, de)?;
    let guard = ContextGuard::pooling_guard(de)?;
    Ok(Utf8PathBuf::from(
      portable_path.into_path_string(guard.project_root()),
    ))
  }
}
