use std::path::Path;

use sugar_path::SugarPath;

use crate::{ContextGuard, Result, cacheable, with::AsConverter};

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
pub struct PortablePath(String);

impl PortablePath {
  pub fn new(path: &Path, project_root: Option<&Path>) -> Self {
    if let Some(project_root) = project_root {
      return Self(path.relative(project_root).to_slash_lossy().into_owned());
    }

    Self(path.to_slash_lossy().into_owned())
  }
  pub fn into_abs_path_string(self, project_root: Option<&Path>) -> String {
    if let Some(project_root) = project_root {
      return self
        .0
        .absolutize_with(project_root)
        .to_string_lossy()
        .into_owned();
    }
    self.0
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
    Ok(T::from(self.into_abs_path_string(guard.project_root())))
  }
}
