use std::path::Path;

use regex::Regex;
use sugar_path::SugarPath;

use crate::{ContextGuard, Result, cacheable, with::AsConverter};

/// A portable string representation that can detect and convert absolute paths within
/// the string to relative paths for cross-platform serialization.
///
/// This implementation uses regex to identify absolute paths based on these rules:
/// - Unix-like paths: Must start with `/` and be preceded by non-letter characters or start of string
/// - Paths are terminated by special characters: `|`, space, `)`, `?`, `=`, `,`, `;`, `#`, `&`
/// - Multiple paths in a single string are all detected and converted
///
/// # Use Cases
///
/// - Module identifiers containing paths: `"ignored|/home/user/project/src/main.rs"`
/// - Cache keys with paths: `"/absolute/path/to/module|hash_value"`
/// - Composite identifiers: `"module_type|/path/file.js|layer"`
/// - Multiple paths: `"ignored|/home/aaa?name=/home/bbb"`
///
/// # Example
///
/// ```rust,ignore
/// // Serialize on Linux (project_root = /home/user/project)
/// let identifier = "ignored|/home/user/project/src/main.rs";
/// // Stored as: "ignored|src/main.rs"
///
/// // Deserialize on Windows (project_root = C:\workspace)
/// // Results in: "ignored|C:\workspace\src\main.rs"
/// ```
#[cacheable(crate=crate, hashable)]
pub struct PortableString {
  /// The string with paths converted to portable format
  content: String,
  /// Records which segments were transformed
  transformations: Vec<PathTransformation>,
}

/// Records a path transformation within the string
#[cacheable(crate=crate, hashable)]
struct PathTransformation {
  /// Starting byte position in the transformed string
  position: usize,
  /// Length of the portable path in bytes
  length: usize,
  /// The portable (relative) path
  portable_path: String,
}

impl PortableString {
  /// Create a portable string, converting absolute paths to relative if project_root is provided
  pub fn new(content: &str, project_root: Option<&Path>) -> Self {
    let Some(project_root) = project_root else {
      return Self {
        content: content.to_string(),
        transformations: Vec::new(),
      };
    };

    let regex = create_path_regex();
    let mut transformations = Vec::new();
    let mut result = String::with_capacity(content.len());
    let mut last_end = 0;

    for cap in regex.captures_iter(content) {
      let full_match = cap.get(0).unwrap();
      let prefix = cap.get(1).unwrap();
      let path_match = cap.get(2).unwrap();

      let path_str = path_match.as_str();

      // Skip if this looks like a URL (additional safety check)
      if is_url_context(content, full_match.start()) {
        result.push_str(&content[last_end..full_match.end()]);
        last_end = full_match.end();
        continue;
      }

      // Add content before this match including the prefix
      result.push_str(&content[last_end..prefix.end()]);

      // Convert to portable format
      let portable_path = make_path_portable(path_str, project_root);
      let transformation_pos = result.len();

      result.push_str(&portable_path);

      transformations.push(PathTransformation {
        position: transformation_pos,
        length: portable_path.len(),
        portable_path: portable_path.clone(),
      });

      last_end = full_match.end();
    }

    // Add remaining content
    result.push_str(&content[last_end..]);

    Self {
      content: result,
      transformations,
    }
  }

  /// Convert back to string, resolving relative paths to absolute using project_root
  pub fn into_string(self, project_root: Option<&Path>) -> String {
    if self.transformations.is_empty() {
      return self.content;
    }

    let Some(project_root) = project_root else {
      return self.content;
    };

    // Reconstruct with absolute paths
    let mut result = String::with_capacity(self.content.len() * 2);
    let mut last_pos = 0;

    for transformation in &self.transformations {
      // Add content before this path
      result.push_str(&self.content[last_pos..transformation.position]);

      // Convert portable path back to absolute
      let absolute_path = restore_absolute_path(&transformation.portable_path, project_root);
      result.push_str(&absolute_path);

      last_pos = transformation.position + transformation.length;
    }

    // Add remaining content
    result.push_str(&self.content[last_pos..]);

    result
  }
}

/// Create a regex pattern for matching absolute paths
fn create_path_regex() -> Regex {
  // Pattern for Unix-like absolute paths:
  // (^|[^a-zA-Z:])           - Capture group 1: Prefix (start of string OR non-letter non-colon char)
  // (/[^| \t\n\r)?=,;#&]+)   - Capture group 2: Path (starts with /, ends at terminators)
  //
  // Terminators: |, whitespace ( \t\n\r), ), ?, =, ,, ;, #, &
  // These are common separators in identifiers and URLs
  //
  // Why exclude letters and colon in prefix:
  // - Letters: Prevents matching in URLs like "http://..." (the "p" before "://" would match)
  // - Colon: Prevents matching URL schemes like "webpack:///path"
  //
  // Windows paths: TODO - to be implemented later

  Regex::new(r"(^|[^a-zA-Z:])(/[^| \t\n\r)?=,;#&]+)").expect("Invalid regex pattern")
}

/// Check if the match is part of a URL (additional safety check)
fn is_url_context(content: &str, match_start: usize) -> bool {
  // Look back to see if there's a :// pattern nearby
  let check_start = match_start.saturating_sub(10);
  let context = &content[check_start..match_start.min(content.len())];

  // Check for URL schemes (://  or webpack:)
  // Also check if the previous character is / (indicating // in a URL)
  context.contains("://")
    || context.ends_with("webpack:")
    || context.ends_with(":")
    || context.ends_with("/")
}

/// Convert an absolute path to a portable relative path
fn make_path_portable(path_str: &str, project_root: &Path) -> String {
  // Normalize path separators for parsing
  let normalized = path_str.replace('\\', "/");
  let path = Path::new(&normalized);

  // Try to make it relative to project_root
  if path.is_absolute() {
    let relative = path.relative(project_root);
    // Always use forward slashes for portability
    relative.to_slash_lossy().into_owned()
  } else {
    // Already relative or invalid, use normalized version
    normalized
  }
}

/// Restore a portable path to absolute using project_root
fn restore_absolute_path(portable_path: &str, project_root: &Path) -> String {
  let path = Path::new(portable_path);

  // If already absolute (shouldn't happen), return as-is
  if path.is_absolute() {
    return portable_path.to_string();
  }

  // Make it absolute with project_root
  let absolute = portable_path.absolutize_with(project_root);
  let absolute_str = absolute.to_string_lossy().into_owned();

  // Keep forward slashes for Unix paths (Windows will be handled separately later)
  absolute_str
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
    Ok(T::from(self.into_string(guard.project_root())))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_unix_path_after_pipe() {
    let s = "ignored|/home/user/project/src/main.rs";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "ignored|src/main.rs");
    assert_eq!(portable.transformations.len(), 1);

    let restored = portable.into_string(Some(project_root));
    assert_eq!(restored, "ignored|/home/user/project/src/main.rs");
  }

  #[test]
  fn test_path_at_start() {
    let s = "/home/user/project/src/app.js";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "src/app.js");

    let restored = portable.into_string(Some(project_root));
    assert_eq!(restored, "/home/user/project/src/app.js");
  }

  #[test]
  fn test_multiple_paths() {
    let s = "ignored|/home/user/project/a.js other|/home/user/project/b.js";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "ignored|a.js other|b.js");
    assert_eq!(portable.transformations.len(), 2);

    let restored = portable.into_string(Some(project_root));
    assert_eq!(
      restored,
      "ignored|/home/user/project/a.js other|/home/user/project/b.js"
    );
  }

  #[test]
  fn test_multiple_paths_with_query_params() {
    let s = "ignored|/home/aaa?name=/home/bbb";
    let project_root = Path::new("/home");

    let portable = PortableString::new(s, Some(project_root));
    // Should match both /home/aaa and /home/bbb
    assert_eq!(portable.content, "ignored|aaa?name=bbb");
    assert_eq!(portable.transformations.len(), 2);

    let restored = portable.into_string(Some(project_root));
    assert_eq!(restored, "ignored|/home/aaa?name=/home/bbb");
  }

  #[test]
  fn test_module_type_pattern() {
    let s = "javascript/auto|/home/user/project/src/app.js|layer1";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "javascript/auto|src/app.js|layer1");

    let restored = portable.into_string(Some(project_root));
    assert_eq!(
      restored,
      "javascript/auto|/home/user/project/src/app.js|layer1"
    );
  }

  #[test]
  fn test_lazy_compilation_proxy() {
    let s = "lazy-compilation-proxy|/home/user/project/src/index.js";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "lazy-compilation-proxy|src/index.js");
  }

  #[test]
  fn test_no_false_positive_for_url() {
    let s = "https://example.com/path/to/file.js";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    // Should not transform URLs (: before / prevents match)
    assert_eq!(portable.content, s);
    assert_eq!(portable.transformations.len(), 0);
  }

  #[test]
  fn test_webpack_protocol() {
    let s = "webpack:///src/main.js";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    // Should not transform webpack:// paths (: prevents match)
    assert_eq!(portable.content, s);
    assert_eq!(portable.transformations.len(), 0);
  }

  #[test]
  fn test_no_project_root() {
    let s = "ignored|/home/user/project/src/main.rs";
    let portable = PortableString::new(s, None);

    assert_eq!(portable.content, s);
    assert_eq!(portable.transformations.len(), 0);
  }

  #[test]
  fn test_no_paths_in_string() {
    let s = "just a regular string without any paths";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, s);
    assert_eq!(portable.transformations.len(), 0);
  }

  #[test]
  fn test_path_after_paren() {
    let s = "remote (scope) /home/user/project/entry.js";
    let project_root = Path::new("/home/user/project");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "remote (scope) entry.js");
    assert_eq!(portable.transformations.len(), 1);

    let restored = portable.into_string(Some(project_root));
    assert_eq!(restored, "remote (scope) /home/user/project/entry.js");
  }

  #[test]
  fn test_unicode_path() {
    let s = "ignored|/home/用户/项目/src/main.js";
    let project_root = Path::new("/home/用户/项目");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "ignored|src/main.js");

    let restored = portable.into_string(Some(project_root));
    assert_eq!(restored, "ignored|/home/用户/项目/src/main.js");
  }

  #[test]
  fn test_comma_separated_paths() {
    let s = "files=/path/a.js,/path/b.js";
    let project_root = Path::new("/path");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "files=a.js,b.js");
    assert_eq!(portable.transformations.len(), 2);

    let restored = portable.into_string(Some(project_root));
    assert_eq!(restored, "files=/path/a.js,/path/b.js");
  }

  #[test]
  fn test_path_with_hash() {
    let s = "module=/path/file.js#hash";
    let project_root = Path::new("/path");

    let portable = PortableString::new(s, Some(project_root));
    assert_eq!(portable.content, "module=file.js#hash");
    assert_eq!(portable.transformations.len(), 1);
  }
}
