use std::path::Path;

use cow_utils::CowUtils;

use crate::{ContextGuard, Result, cacheable, with::AsConverter};

// Escape literal NULs before inserting root markers, including when portability is disabled.
// The escape has no trailing NUL, so escaped data cannot form a root marker.
const ESCAPE: &str = "\0escape";

/// Distinct markers preserve Windows separator forms.
const ROOT: &str = "\0root/\0";
#[cfg(windows)]
const ROOT_NATIVE: &str = "\0root\\\0";
#[cfg(windows)]
const ROOT_ESCAPED: &str = "\0root\\\\\0";

/// Replaces exact root prefixes in identifiers while preserving paths outside the root.
#[cacheable(crate=crate, hashable)]
pub struct PortableString {
  content: String,
}

impl PortableString {
  pub fn new(content: &str, guard: &ContextGuard) -> Self {
    let content = content.cow_replace('\0', ESCAPE);
    let project_root = guard.project_root();
    let Some(project_root) = project_root else {
      return Self {
        content: content.into_owned(),
      };
    };
    for part in content.split(['|', '!', '?', '=', ' ', '\t', '\n', '\r']) {
      let path = Path::new(part);
      if path.is_absolute() && !path.starts_with(project_root) {
        guard.mark_outside_root();
      }
    }
    let root = project_root.to_string_lossy();
    #[cfg(windows)]
    let content = {
      let native = format!("{}\\", root.cow_replace('/', "\\").trim_end_matches('\\'));
      content
        .cow_replace(native.cow_replace('\\', "\\\\").as_ref(), ROOT_ESCAPED)
        .cow_replace(&native, ROOT_NATIVE)
        .into_owned()
    };
    #[cfg(windows)]
    let root = root.cow_replace('\\', "/");
    Self {
      content: content
        .cow_replace(&format!("{}/", root.trim_end_matches('/')), ROOT)
        .into_owned(),
    }
  }

  pub fn into_path_string(self, project_root: Option<&Path>) -> String {
    let Some(project_root) = project_root else {
      return self.content.cow_replace(ESCAPE, "\0").into_owned();
    };
    let root = project_root.to_string_lossy();
    #[cfg(windows)]
    let content = {
      let native = format!("{}\\", root.cow_replace('/', "\\").trim_end_matches('\\'));
      self
        .content
        .cow_replace(ROOT_ESCAPED, &native.cow_replace('\\', "\\\\"))
        .cow_replace(ROOT_NATIVE, &native)
        .into_owned()
    };
    #[cfg(not(windows))]
    let content = self.content;
    #[cfg(windows)]
    let root = root.cow_replace('\\', "/");
    content
      .cow_replace(ROOT, &format!("{}/", root.trim_end_matches('/')))
      .cow_replace(ESCAPE, "\0")
      .into_owned()
  }
}

impl<T> AsConverter<T> for PortableString
where
  T: From<String> + AsRef<str>,
{
  fn serialize(data: &T, guard: &ContextGuard) -> Result<Self>
  where
    Self: Sized,
  {
    Ok(Self::new(data.as_ref(), guard))
  }

  fn deserialize(self, guard: &ContextGuard) -> Result<T> {
    Ok(T::from(self.into_path_string(guard.project_root())))
  }
}
