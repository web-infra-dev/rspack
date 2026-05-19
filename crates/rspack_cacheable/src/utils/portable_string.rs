use std::{path::Path, sync::LazyLock};

use regex::Regex;
use sugar_path::SugarPath;

use crate::{ContextGuard, Result, cacheable, with::AsConverter};

static PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  #[cfg(windows)]
  {
    Regex::new(r"(^|[^a-zA-Z0-9./\\])([a-zA-Z]:\\([^/\\ \t\n\r?=,;:#&|!]+\\)*)")
      .expect("Invalid regex pattern")
  }
  #[cfg(not(windows))]
  {
    Regex::new(r"(^|[^a-zA-Z0-9./\\])(/([^/\\ \t\n\r?=,;:#&|!]+/)*)")
      .expect("Invalid regex pattern")
  }
});

const PROJECT_ROOT_PLACEHOLDER: &str = "<project_root>";

static PROJECT_ROOT_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  let pattern = format!(
    r"{}/(([^/\\ \t\n\r?=,;:#&|!]+/)*)",
    regex::escape(PROJECT_ROOT_PLACEHOLDER)
  );
  Regex::new(&pattern).expect("Invalid regex pattern")
});

/// A portable string representation that can detect and convert absolute paths within
/// the string to portable format using `<project_root>` placeholder for cross-platform serialization.
///
/// # Serialization Strategy
///
/// Absolute paths are replaced with `<project_root>` prefix:
/// - `/home/user/project/src/` → `<project_root>/src/`
/// - `C:\Users\project\src\` → `<project_root>/src/`
///
/// Paths outside project_root use relative paths with `..`:
/// - `/home/user/other/lib/` → `<project_root>/../other/lib/`
///
/// # Deserialization Strategy
///
/// Replace `<project_root>` with actual project_root and normalize:
/// - `<project_root>/src/` + `/home/user/project` → `/home/user/project/src/`
/// - `<project_root>/../other/lib/` + `/home/user/project` → `/home/user/other/lib/`
///
/// # Use Cases
///
/// - Module identifiers containing paths: `"ignored|/home/user/project/src/"`
/// - Cache keys with paths: `"/absolute/path/to/module/|hash_value"`
/// - Composite identifiers: `"module_type|/path/file/|layer"`
/// - Multiple paths: `"ignored|/home/aaa/?name=/home/bbb/"`
///
/// # Example
///
/// ```rust,ignore
/// // Serialize on Linux (project_root = /home/user/project)
/// let identifier = "ignored|/home/user/project/src/";
/// // Stored as: "ignored|<project_root>/src/"
///
/// // Deserialize on Windows (project_root = C:\workspace)
/// // Results in: "ignored|C:\workspace\src\"
/// ```
#[cacheable(crate=crate, hashable)]
pub struct PortableString {
  /// The string with paths converted to portable format using <project_root> placeholder
  content: String,
}

impl PortableString {
  /// Create a portable string, converting absolute paths to use <project_root> placeholder
  pub fn new(content: &str, project_root: Option<&Path>) -> Self {
    let Some(project_root) = project_root else {
      return Self {
        content: content.to_string(),
      };
    };

    let mut result = String::with_capacity(content.len());
    let mut last_end = 0;

    for cap in PATH_REGEX.captures_iter(content) {
      let path_match = cap.get(2).expect("PATH_REGEX should have capture group 2");

      result.push_str(&content[last_end..path_match.start()]);

      // Convert to portable format with <project_root> placeholder
      // sugar_path 2.x relative() fast path may preserve trailing slash (e.g. "src/"),
      // so trim it before appending "/" to avoid double slashes like "<project_root>/src//"
      let relative_path = path_match
        .as_str()
        .relative(project_root)
        .to_slash_lossy()
        .trim_end_matches('/')
        .to_string();

      let portable_path = if relative_path.is_empty() || relative_path == "." {
        format!("{PROJECT_ROOT_PLACEHOLDER}/")
      } else {
        format!("{PROJECT_ROOT_PLACEHOLDER}/{relative_path}/")
      };
      result.push_str(&portable_path);

      last_end = path_match.end();
    }

    // Add remaining content
    result.push_str(&content[last_end..]);

    Self { content: result }
  }

  /// Convert back to string, replacing <project_root> with actual project_root
  pub fn into_path_string(self, project_root: Option<&Path>) -> String {
    let Some(project_root) = project_root else {
      return self.content;
    };

    if !self.content.contains(PROJECT_ROOT_PLACEHOLDER) {
      return self.content;
    }

    let content = &self.content;
    let project_root_str = project_root.to_slash_lossy();
    let mut result = String::with_capacity(content.len());
    let mut last_end = 0;

    for cap in PROJECT_ROOT_PATH_REGEX.captures_iter(content) {
      let full_match = cap
        .get(0)
        .expect("PROJECT_ROOT_PATH_REGEX should have capture group 0");
      let relative_match = cap
        .get(1)
        .expect("PROJECT_ROOT_PATH_REGEX should have capture group 1");

      // Add content before this match including the prefix
      result.push_str(&content[last_end..full_match.start()]);

      // Convert to absolute path
      let abs_path_str = format!("{}/{}", &project_root_str, relative_match.as_str());
      let abs_path = abs_path_str.normalize();

      result.push_str(&abs_path.to_string_lossy());
      if !abs_path.ends_with(std::path::MAIN_SEPARATOR_STR) {
        result.push(std::path::MAIN_SEPARATOR);
      }
      last_end = full_match.end();
    }

    result.push_str(&content[last_end..]);

    result
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
    Ok(Self::new(data.as_ref(), guard.project_root()))
  }

  fn deserialize(self, guard: &ContextGuard) -> Result<T> {
    Ok(T::from(self.into_path_string(guard.project_root())))
  }
}
