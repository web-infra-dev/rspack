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
use crate::{ContextGuard, Error};

const PROJECT_ROOT_PLACEHOLDER: &str = "<project_root>";

pub struct PathResolver {
  inner: StringResolver,
  path: String,
}

#[inline]
fn serialize_path(path: &Utf8PathBuf, project_root: Option<&std::path::Path>) -> String {
  if let Some(root) = project_root {
    // Try to strip the project_root prefix
    if let Ok(relative) = path.as_std_path().strip_prefix(root) {
      // Convert to Utf8Path for consistent representation
      if let Some(relative_utf8) = camino::Utf8Path::from_path(relative) {
        return format!("{}/{}", PROJECT_ROOT_PLACEHOLDER, relative_utf8);
      }
    }
  }
  // Fallback: use absolute path
  path.to_string()
}

#[inline]
fn deserialize_path(
  serialized: &str,
  project_root: Option<&std::path::Path>,
) -> Result<Utf8PathBuf, Error> {
  if let Some(relative) = serialized.strip_prefix(PROJECT_ROOT_PLACEHOLDER) {
    // Remove leading slash if present
    let relative = relative.strip_prefix('/').unwrap_or(relative);

    if let Some(root) = project_root {
      // Join with current project_root
      let root_utf8 = camino::Utf8Path::from_path(root)
        .ok_or_else(|| Error::MessageError("project_root is not valid UTF-8"))?;
      return Ok(root_utf8.join(relative));
    } else {
      // No project_root available, can't restore relative path
      return Err(Error::MessageError(
        "cannot deserialize relative path without project_root context",
      ));
    }
  }

  // Absolute path, use as-is
  Ok(Utf8PathBuf::from(serialized))
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
    let project_root = guard.context().project_root();
    let path = serialize_path(field, project_root);

    Ok(PathResolver {
      inner: ArchivedString::serialize_from_str(path.as_str(), serializer)?,
      path,
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
    let project_root = guard.context().project_root();
    deserialize_path(field.as_str(), project_root)
  }
}
