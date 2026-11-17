use std::path::{Component, Path};

use sugar_path::SugarPath;

use crate::{ContextGuard, Result, RootSpelling, cacheable, with::AsConverter};

/// A portable path representation that can be serialized and deserialized across different
/// environments with different project roots.
///
/// When serializing with a `project_root`, absolute paths under either spelling of that root are
/// converted to relative paths. When deserializing, relative paths are resolved back to absolute
/// paths under the same spelling of the current root.
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
  /// The root spelling `path` is relative to, or `None` when `path` is stored verbatim.
  root: Option<RootSpelling>,
}

impl PortablePath {
  /// Create a portable path. Passing a root enables portable conversion; otherwise the native
  /// path representation is preserved. An absolute path left verbatim marks the entry as
  /// outside the root.
  pub fn new(path: &Path, guard: &ContextGuard) -> Self {
    if path.is_absolute() {
      for (spelling, root) in guard.root_spellings() {
        if let Ok(relative) = path.strip_prefix(root)
          && !relative.components().any(|c| c == Component::ParentDir)
        {
          return Self {
            path: relative.to_slash_lossy().into_owned(),
            root: Some(spelling),
          };
        }
      }
      if guard.project_root().is_some() {
        guard.mark_outside_root();
      }
    }

    Self {
      path: path.to_string_lossy().into_owned(),
      root: None,
    }
  }

  /// Convert back to a path string, joining a relative path onto the current root in the spelling
  /// it was stored under.
  pub fn into_path_string(self, guard: &ContextGuard) -> String {
    if let Some(root) = self.root.and_then(|spelling| guard.root(spelling)) {
      return self
        .path
        .absolutize_with(root)
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
    Ok(Self::new(data.as_ref(), guard))
  }

  fn deserialize(self, guard: &ContextGuard) -> Result<T> {
    Ok(T::from(self.into_path_string(guard)))
  }
}
