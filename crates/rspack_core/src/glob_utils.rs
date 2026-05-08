use core::fmt;
use std::sync::Arc;

use async_recursion::async_recursion;
use cow_utils::CowUtils;
use fast_glob::glob_match;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Clone)]
pub struct GlobMatchOptions {
  pub case_sensitive: bool,
  pub require_literal_leading_dot: bool,
}

impl Default for GlobMatchOptions {
  fn default() -> Self {
    Self {
      case_sensitive: true,
      require_literal_leading_dot: true,
    }
  }
}

impl fmt::Display for GlobMatchOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "GlobMatchOptions {{ case_sensitive: {}, require_literal_leading_dot: {} }}",
      self.case_sensitive, self.require_literal_leading_dot
    )
  }
}

/// Escape special glob characters in a literal path string.
/// Replaces `glob::Pattern::escape`.
pub fn escape_glob_pattern(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  for c in s.chars() {
    match c {
      '*' | '?' | '[' | ']' | '{' | '}' => {
        result.push('\\');
        result.push(c);
      }
      _ => result.push(c),
    }
  }
  result
}

/// Match a path against a glob pattern with options.
pub fn glob_match_with_options(pattern: &str, path: &str, options: &GlobMatchOptions) -> bool {
  if options.case_sensitive {
    glob_match(pattern.as_bytes(), path.as_bytes())
  } else {
    let pattern = pattern.to_lowercase();
    let path = path.to_lowercase();
    glob_match(pattern.as_bytes(), path.as_bytes())
  }
}

/// Extract the base directory from a glob pattern.
/// Returns everything before the first glob metacharacter, up to and including the last `/`.
fn extract_glob_base_dir(pattern: &str) -> &str {
  let idx = pattern
    .find(|c: char| ['*', '?', '[', '{'].contains(&c))
    .unwrap_or(pattern.len());
  let before = &pattern[..idx];
  match before.rfind('/') {
    Some(slash_idx) => &pattern[..=slash_idx],
    None => "./",
  }
}

/// Normalize backslashes to forward slashes in a path string.
fn normalize_path_separators(s: &str) -> String {
  s.cow_replace('\\', "/").into_owned()
}

/// Walk a directory tree recursively, calling `on_file` for each file found.
///
/// - `root`: starting directory
/// - `recursive`: whether to descend into subdirectories
/// - `skip_dotfiles`: whether to skip files whose name starts with `.`
/// - `on_file`: called with (full_path, filename) for each file
#[async_recursion]
pub(crate) async fn walk_dir(
  root: &Utf8Path,
  fs: Arc<dyn ReadableFileSystem>,
  recursive: bool,
  skip_dotfiles: bool,
  on_file: &mut (impl FnMut(Utf8PathBuf, String) + Send),
) -> Result<()> {
  if !fs.metadata(root).await.is_ok_and(|m| m.is_directory) {
    return Ok(());
  }
  for filename in fs.read_dir(root).await? {
    let path = root.join(&filename);
    if fs.metadata(&path).await.is_ok_and(|m| m.is_directory) {
      if recursive {
        walk_dir(&path, fs.clone(), recursive, skip_dotfiles, on_file).await?;
      }
    } else if skip_dotfiles && filename.starts_with('.') {
      // skip dotfiles
    } else {
      on_file(path, filename);
    }
  }
  Ok(())
}

/// Find files matching a glob pattern by traversing the filesystem.
/// Replaces `glob::glob_with`.
pub async fn find_files_by_glob(
  pattern: &str,
  options: &GlobMatchOptions,
  fs: Arc<dyn ReadableFileSystem>,
) -> Result<Vec<Utf8PathBuf>> {
  let normalized_pattern = normalize_path_separators(pattern);
  let base_dir = extract_glob_base_dir(&normalized_pattern);
  let base_dir_path = Utf8Path::new(base_dir);

  let mut results = Vec::new();
  walk_dir(
    base_dir_path,
    fs,
    true,  // always recursive for glob
    false, // dotfile filtering handled in callback below
    &mut |path, _filename| {
      if options.require_literal_leading_dot
        && path_has_dot_component(&path, base_dir_path)
        && !pattern_has_explicit_dot_for(&normalized_pattern, base_dir_path, &path)
      {
        return;
      }
      let normalized_path = normalize_path_separators(path.as_str());
      if glob_match_with_options(&normalized_pattern, &normalized_path, options) {
        results.push(path);
      }
    },
  )
  .await?;
  Ok(results)
}

fn path_has_dot_component(path: &Utf8Path, base_dir: &Utf8Path) -> bool {
  let relative = path.strip_prefix(base_dir).unwrap_or(path);
  for component in relative.components() {
    if component.as_str().starts_with('.') {
      return true;
    }
  }
  false
}

/// Check whether the glob pattern has an explicit `.` for a given dot-file path.
fn pattern_has_explicit_dot_for(pattern: &str, base_dir: &Utf8Path, path: &Utf8Path) -> bool {
  let path_str = normalize_path_separators(path.as_str());
  let base_str = normalize_path_separators(base_dir.as_str());

  let relative = if let Some(stripped) = path_str.strip_prefix(&base_str) {
    stripped
  } else {
    &path_str
  };

  for component in relative.split('/') {
    if component.starts_with('.') {
      let dot_component = format!(".{}", &component[1..]);
      if pattern.contains(&format!("/{}", dot_component)) || pattern.starts_with(&dot_component) {
        return true;
      }
    }
  }
  false
}
