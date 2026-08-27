use std::path::Path;

use sugar_path::SugarPath;

use crate::{ContextGuard, Result, cacheable, with::AsConverter};

/// Turns the slash-separated stored form back into the platform's own spelling.
#[cfg(windows)]
#[inline]
fn to_native_separator(path: String) -> String {
  path.replace('/', "\\")
}

/// The stored form already uses this platform's separator.
#[cfg(not(windows))]
#[inline]
fn to_native_separator(path: String) -> String {
  path
}

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
  /// Create a portable path, converting to relative if both path and project_root are absolute
  pub fn new(path: &Path, project_root: Option<&Path>) -> Self {
    if path.is_absolute()
      && let Some(project_root) = project_root
    {
      return Self {
        path: path.relative(project_root).to_slash_lossy().into_owned(),
        transformed: true,
      };
    }

    Self {
      path: path.to_slash_lossy().into_owned(),
      transformed: false,
    }
  }

  /// Convert back to path string using project_root if the path was transformed
  pub fn into_path_string(self, project_root: Option<&Path>) -> String {
    if self.transformed
      && let Some(project_root) = project_root
    {
      // `absolutize_with` normalizes, so the result already uses the native separator.
      return self
        .path
        .absolutize_with(project_root)
        .to_string_lossy()
        .into_owned();
    }
    // Paths are always stored slash-separated, so an untransformed one has to be put back into
    // the native form. Restoring `D:/a/b` where the rest of the build produces `D:\a\b` yields
    // two spellings of one path, and consumers that compare paths as strings (watchpack) then
    // fail to match the restored one.
    to_native_separator(self.path)
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
