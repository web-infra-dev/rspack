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
    let pattern = pattern.cow_to_lowercase();
    let path = path.cow_to_lowercase();
    glob_match(pattern.as_bytes(), path.as_bytes())
  }
}

/// Extract the base directory from a glob pattern.
/// Returns everything before the first glob metacharacter, up to and including the last `/`.
fn extract_glob_base_dir(pattern: &str) -> &str {
  let mut escaped = false;
  let mut idx = pattern.len();
  for (byte_idx, c) in pattern.char_indices() {
    if escaped {
      escaped = false;
      continue;
    }

    if c == '\\' {
      escaped = true;
      continue;
    }

    if ['*', '?', '[', '{'].contains(&c) {
      idx = byte_idx;
      break;
    }
  }

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
  should_enter_dir: &mut (impl FnMut(&Utf8Path) -> bool + Send),
  on_file: &mut (impl FnMut(Utf8PathBuf, String) + Send),
) -> Result<()> {
  if !fs.metadata(root).await.is_ok_and(|m| m.is_directory) {
    return Ok(());
  }
  for filename in fs.read_dir(root).await? {
    let path = root.join(&filename);
    if fs.metadata(&path).await.is_ok_and(|m| m.is_directory) {
      if recursive && should_enter_dir(&path) {
        walk_dir(
          &path,
          fs.clone(),
          recursive,
          skip_dotfiles,
          should_enter_dir,
          on_file,
        )
        .await?;
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
    &mut |_path| true,
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
  let base_str = normalize_path_separators(base_dir.as_str());
  let path_str = normalize_path_separators(path.as_str());
  let pattern_suffix = pattern.strip_prefix(&base_str).unwrap_or(pattern);

  let relative = path_str.strip_prefix(&base_str).unwrap_or(&path_str);
  let pattern_segments = pattern_suffix
    .split('/')
    .filter(|segment| !segment.is_empty())
    .collect::<Vec<_>>();
  let path_segments = relative
    .split('/')
    .filter(|segment| !segment.is_empty())
    .collect::<Vec<_>>();

  fn matches_explicit_dot_segments(patterns: &[&str], paths: &[&str]) -> bool {
    match (patterns.split_first(), paths.split_first()) {
      (None, None) => true,
      (None, Some(_)) => false,
      (Some((&"**", pattern_rest)), _) => {
        matches_explicit_dot_segments(pattern_rest, paths)
          || matches!(
            paths.split_first(),
            Some((&path_head, path_rest))
              if !path_head.starts_with('.') && matches_explicit_dot_segments(patterns, path_rest)
          )
      }
      (Some((&pattern_head, pattern_rest)), Some((&path_head, path_rest))) => {
        if path_head.starts_with('.') && !pattern_head.starts_with('.') {
          return false;
        }
        glob_match(pattern_head.as_bytes(), path_head.as_bytes())
          && matches_explicit_dot_segments(pattern_rest, path_rest)
      }
      (Some(_), None) => false,
    }
  }

  matches_explicit_dot_segments(&pattern_segments, &path_segments)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extract_glob_base_dir_skips_escaped_metacharacters() {
    assert_eq!(
      extract_glob_base_dir("./fixtures/a\\[b\\]/file"),
      "./fixtures/a\\[b\\]/"
    );
    assert_eq!(
      extract_glob_base_dir("./fixtures/a\\[b\\]/**/*.js"),
      "./fixtures/a\\[b\\]/"
    );
  }

  #[test]
  fn explicit_dot_patterns_allow_wildcard_dot_segments() {
    let base_dir = Utf8Path::new("./fixtures/");

    assert!(pattern_has_explicit_dot_for(
      "./fixtures/**/.*",
      base_dir,
      Utf8Path::new("./fixtures/.env")
    ));
    assert!(pattern_has_explicit_dot_for(
      "./fixtures/**/.*/index.js",
      base_dir,
      Utf8Path::new("./fixtures/.cache/index.js")
    ));
    assert!(!pattern_has_explicit_dot_for(
      "./fixtures/**/index.js",
      base_dir,
      Utf8Path::new("./fixtures/.cache/index.js")
    ));
  }
}
