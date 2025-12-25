use camino::Utf8PathBuf;
use rkyv::{
  Place,
  de::Pooling,
  rancor::Fallible,
  ser::{Sharing, Writer},
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{ContextGuard, Error, utils::PortablePath, with::AsConverter};

pub struct PathResolver {
  inner: StringResolver,
  path: String,
}

impl ArchiveWith<Utf8PathBuf> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = PathResolver;

  #[inline]
  fn resolve_with(_field: &Utf8PathBuf, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let PathResolver { inner, path } = resolver;
    ArchivedString::resolve_from_str(path.as_str(), inner, out);
  }
}

impl<S> SerializeWith<Utf8PathBuf, S> for AsPreset
where
  S: ?Sized + Fallible<Error = Error> + Writer + Sharing<Error>,
{
  #[inline]
  fn serialize_with(field: &Utf8PathBuf, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let guard = ContextGuard::sharing_guard(serializer)?;
    // Use PortablePath to serialize, which handles project_root and path normalization
    let portable_path = PortablePath::serialize(field, guard)?;

    Ok(PathResolver {
      inner: ArchivedString::serialize_from_str(portable_path.0.as_str(), serializer)?,
      path: portable_path.0,
    })
  }
}

impl<D> DeserializeWith<ArchivedString, Utf8PathBuf, D> for AsPreset
where
  D: ?Sized + Fallible<Error = Error> + Pooling<Error>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, de: &mut D) -> Result<Utf8PathBuf, D::Error> {
    let guard = ContextGuard::pooling_guard(de)?;
    let portable = PortablePath(field.to_string());
    portable.deserialize(guard)
  }
}
