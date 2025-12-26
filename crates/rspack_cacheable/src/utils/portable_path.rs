use std::path::Path;

use crate::{ContextGuard, Error, Result, cacheable, with::AsConverter};

const PROJECT_ROOT_PLACEHOLDER: &str = "<project_root>";

/// A portable path representation that can be serialized and deserialized across different
/// environments with different project roots.
///
/// # Usage
///
/// Use with `with::As<PortablePath>` to make path fields portable:
///
/// ```rust,ignore
/// use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
/// use std::path::PathBuf;
///
/// #[cacheable]
/// struct MyStruct {
///   #[cacheable(with=As<PortablePath>)]
///   path: PathBuf,
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// // Serialize on Linux with project_root = /home/user/project
/// let data = MyStruct {
///   path: PathBuf::from("/home/user/project/src/main.rs"),
/// };
/// let bytes = to_bytes(&data, &context).unwrap();
///
/// // Deserialize on Windows with project_root = C:\workspace
/// let restored: MyStruct = from_bytes(&bytes, &windows_context).unwrap();
/// // restored.path == PathBuf::from("C:\\workspace\\src\\main.rs")
/// ```
#[cacheable(crate=crate, hashable)]
pub struct PortablePath(pub String);

impl<T> AsConverter<T> for PortablePath
where
  T: From<String> + AsRef<Path>,
{
  fn serialize(data: &T, guard: &ContextGuard) -> Result<Self>
  where
    Self: Sized,
  {
    let path = data.as_ref();
    let project_root = guard.context().project_root();
    if let Some(root) = project_root {
      // Try to strip the project_root prefix
      if let Ok(relative) = path.strip_prefix(root) {
        // Convert to string representation
        if let Some(relative_str) = relative.to_str() {
          // Normalize path separators to forward slashes for portability
          let normalized = relative_str.replace('\\', "/");
          return Ok(Self(format!("{}/{}", PROJECT_ROOT_PLACEHOLDER, normalized)));
        }
      }
    }
    // Fallback: use absolute path, also normalize separators for portability
    let path_str = path.to_string_lossy();
    let normalized = path_str.replace('\\', "/");
    Ok(Self(normalized))
  }

  fn deserialize(self, guard: &ContextGuard) -> Result<T> {
    let path = self.0;
    let project_root = guard.context().project_root();
    if let Some(relative) = path.strip_prefix(PROJECT_ROOT_PLACEHOLDER) {
      // Remove leading slash if present
      let relative = relative.strip_prefix('/').unwrap_or(relative);

      if let Some(root) = project_root {
        // Join with current project_root
        // Note: Path::join() handles forward slashes correctly on all platforms
        let path = root.join(relative);
        return Ok(T::from(path.to_string_lossy().into_owned()));
      } else {
        // No project_root available, can't restore relative path
        return Err(Error::MessageError(
          "cannot deserialize relative path without project_root context",
        ));
      }
    }

    // Absolute path, use as-is
    // Note: PathBuf::from() accepts forward slashes on all platforms
    Ok(T::from(path.to_string()))
  }
}
