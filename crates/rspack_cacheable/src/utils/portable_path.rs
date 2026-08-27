use std::path::Path;

use sugar_path::SugarPath;

use crate::{ContextGuard, Result, cacheable, with::AsConverter};

/// A portable path representation that can be serialized and deserialized across different
/// environments with different project roots.
///
/// When serializing with a `project_root`, absolute paths are converted to relative paths.
/// When deserializing, relative paths are resolved back to absolute paths.
///
/// # Example
///
/// ```rust,ignore
/// // Serialize on Linux (project_root = /home/user/project)
/// let path = PathBuf::from("/home/user/project/src/main.rs");
/// // Stored as: "src/main.rs" (relative)
///
/// // Deserialize on Windows (project_root = C:\workspace)
/// // Results in: "C:\workspace\src\main.rs"
/// ```
#[cacheable(crate=crate, hashable)]
pub struct PortablePath {
  path: String,
  /// Whether the path was transformed to relative during serialization
  transformed: bool,
}

impl PortablePath {
  /// Create a portable path. Passing a root enables portable conversion; otherwise the native
  /// path representation is preserved.
  pub fn new(path: &Path, portable_project_root: Option<&Path>) -> Self {
    if path.is_absolute()
      && let Some(portable_project_root) = portable_project_root
    {
      return Self {
        path: path
          .relative(portable_project_root)
          .to_slash_lossy()
          .into_owned(),
        transformed: true,
      };
    }

    Self {
      path: if portable_project_root.is_some() {
        path.to_slash_lossy().into_owned()
      } else {
        path.to_string_lossy().into_owned()
      },
      transformed: false,
    }
  }

  /// Convert back to a path string using the portable project root if the path was transformed.
  pub fn into_path_string(self, portable_project_root: Option<&Path>) -> String {
    if self.transformed
      && let Some(portable_project_root) = portable_project_root
    {
      return self
        .path
        .absolutize_with(portable_project_root)
        .to_string_lossy()
        .into_owned();
    }
    self.path
  }
}

impl<T> AsConverter<T> for PortablePath
where
  T: From<String> + AsRef<Path>,
{
  fn serialize(data: &T, guard: &ContextGuard) -> Result<Self>
  where
    Self: Sized,
  {
    Ok(Self::new(data.as_ref(), guard.project_root()))
  }

  fn deserialize(self, guard: &ContextGuard) -> Result<T> {
    Ok(T::from(self.into_path_string(guard.project_root())))
  }
}
