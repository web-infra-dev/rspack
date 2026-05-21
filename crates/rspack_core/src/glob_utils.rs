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

/// Match a path against a glob pattern while respecting the `require_literal_leading_dot` option.
pub fn glob_match_with_explicit_dot(
  pattern: &str,
  path: &str,
  base_dir: &str,
  options: &GlobMatchOptions,
) -> bool {
  let normalized_pattern = normalize_path_separators(pattern);
  let normalized_path = normalize_path_separators(path);
  let normalized_base_dir = normalize_path_separators(base_dir);

  glob_match_normalized_with_explicit_dot(
    &normalized_pattern,
    &normalized_path,
    &normalized_base_dir,
    options,
  )
}

/// Match normalized path strings against a normalized glob pattern.
pub(crate) fn glob_match_normalized_with_explicit_dot(
  normalized_pattern: &str,
  normalized_path: &str,
  normalized_base_dir: &str,
  options: &GlobMatchOptions,
) -> bool {
  if options.require_literal_leading_dot
    && path_has_dot_component(normalized_path, normalized_base_dir)
    && !pattern_has_explicit_dot_for(
      normalized_pattern,
      normalized_base_dir,
      normalized_path,
      options,
    )
  {
    return false;
  }

  glob_match_with_options(normalized_pattern, normalized_path, options)
}

/// Return whether a character has special meaning in glob patterns.
pub fn is_glob_metacharacter(c: char) -> bool {
  matches!(c, '*' | '?' | '[' | '{')
}

/// Return the byte index after the base directory prefix of a glob pattern.
pub fn glob_base_dir_end(pattern: &str) -> usize {
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

    if is_glob_metacharacter(c) {
      idx = byte_idx;
      break;
    }
  }

  pattern[..idx]
    .rfind('/')
    .map_or(0, |slash_idx| slash_idx + 1)
}

/// Extract the base directory from a glob pattern.
/// Returns everything before the first glob metacharacter, up to and including the last `/`.
pub fn extract_glob_base_dir(pattern: &str) -> &str {
  match glob_base_dir_end(pattern) {
    0 => "./",
    end => &pattern[..end],
  }
}

/// Normalize backslashes to forward slashes in a path string.
pub fn normalize_path_separators(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\' {
      if chars
        .peek()
        .is_some_and(|next| matches!(next, '*' | '?' | '[' | ']' | '{' | '}'))
      {
        result.push(c);
      } else {
        result.push('/');
      }
    } else {
      result.push(c);
    }
  }
  result
}

fn unescape_glob_path(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\'
      && chars
        .peek()
        .is_some_and(|next| matches!(next, '*' | '?' | '[' | ']' | '{' | '}'))
    {
      if let Some(next) = chars.next() {
        result.push(next);
      }
    } else {
      result.push(c);
    }
  }
  result
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
  let unescaped_base_dir = unescape_glob_path(base_dir);
  let base_dir_path = Utf8Path::new(&unescaped_base_dir);

  let mut results = Vec::new();
  walk_dir(
    base_dir_path,
    fs,
    true,  // always recursive for glob
    false, // dotfile filtering handled in callback below
    &mut |_path| true,
    &mut |path, _filename| {
      if glob_match_with_explicit_dot(
        &normalized_pattern,
        path.as_str(),
        base_dir_path.as_str(),
        options,
      ) {
        results.push(path);
      }
    },
  )
  .await?;
  Ok(results)
}

fn path_has_dot_component(path: &str, base_dir: &str) -> bool {
  let relative = path.strip_prefix(base_dir).unwrap_or(path);
  for component in relative.split('/').filter(|segment| !segment.is_empty()) {
    if component.starts_with('.') {
      return true;
    }
  }
  false
}

/// Check whether the glob pattern has an explicit `.` for a given dot-file path.
fn pattern_has_explicit_dot_for(
  pattern: &str,
  base_dir: &str,
  path: &str,
  options: &GlobMatchOptions,
) -> bool {
  let pattern_suffix = pattern.strip_prefix(base_dir).unwrap_or(pattern);

  let relative = path.strip_prefix(base_dir).unwrap_or(path);
  let pattern_segments = pattern_suffix
    .split('/')
    .filter(|segment| !segment.is_empty())
    .collect::<Vec<_>>();
  let path_segments = relative
    .split('/')
    .filter(|segment| !segment.is_empty())
    .collect::<Vec<_>>();

  fn matches_explicit_dot_segments(
    patterns: &[&str],
    paths: &[&str],
    options: &GlobMatchOptions,
  ) -> bool {
    match (patterns.split_first(), paths.split_first()) {
      (None, None) => true,
      (None, Some(_)) => false,
      (Some((&"**", pattern_rest)), _) => {
        matches_explicit_dot_segments(pattern_rest, paths, options)
          || matches!(
            paths.split_first(),
            Some((&path_head, path_rest))
              if !path_head.starts_with('.') && matches_explicit_dot_segments(patterns, path_rest, options)
          )
      }
      (Some((&pattern_head, pattern_rest)), Some((&path_head, path_rest))) => {
        if path_head.starts_with('.') && !pattern_head.starts_with('.') {
          return false;
        }
        glob_match_with_options(pattern_head, path_head, options)
          && matches_explicit_dot_segments(pattern_rest, path_rest, options)
      }
      (Some(_), None) => false,
    }
  }

  matches_explicit_dot_segments(&pattern_segments, &path_segments, options)
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
    assert_eq!(
      extract_glob_base_dir("./fixtures/file\\*.js"),
      "./fixtures/"
    );
    assert_eq!(
      extract_glob_base_dir("./fixtures/directory\\?1/**/*.js"),
      "./fixtures/directory\\?1/"
    );
  }

  #[test]
  fn normalize_path_separators_preserves_glob_escapes() {
    assert_eq!(
      normalize_path_separators("./fixtures/a\\[b\\]/**/*.js"),
      "./fixtures/a\\[b\\]/**/*.js"
    );
    assert_eq!(
      normalize_path_separators("./fixtures/file\\*.js"),
      "./fixtures/file\\*.js"
    );
    assert_eq!(
      normalize_path_separators("./fixtures/file\\?.js"),
      "./fixtures/file\\?.js"
    );
    assert_eq!(
      normalize_path_separators("C:\\fixtures\\a\\[b\\]\\file.js"),
      "C:/fixtures/a\\[b\\]/file.js"
    );
    assert_eq!(
      normalize_path_separators("C:\\repo\\src/*.js"),
      "C:/repo/src/*.js"
    );
  }

  #[test]
  fn unescape_glob_path_restores_literal_path_segments() {
    assert_eq!(
      unescape_glob_path("./fixtures/a\\[b\\]/"),
      "./fixtures/a[b]/"
    );
    assert_eq!(
      unescape_glob_path("./fixtures/file\\*.js"),
      "./fixtures/file*.js"
    );
    assert_eq!(
      unescape_glob_path("./fixtures/directory\\?1/"),
      "./fixtures/directory?1/"
    );
  }

  #[test]
  fn escaped_star_and_question_match_literal_path_segments() {
    let options = GlobMatchOptions::default();

    assert!(glob_match_with_options(
      "./fixtures/file\\*.js",
      "./fixtures/file*.js",
      &options
    ));
    assert!(!glob_match_with_options(
      "./fixtures/file\\*.js",
      "./fixtures/file-a.js",
      &options
    ));
    assert!(glob_match_with_options(
      "./fixtures/directory\\?1/**/*.js",
      "./fixtures/directory?1/index.js",
      &options
    ));
    assert!(!glob_match_with_options(
      "./fixtures/directory\\?1/**/*.js",
      "./fixtures/directory-a1/index.js",
      &options
    ));
  }

  #[test]
  fn explicit_dot_patterns_allow_wildcard_dot_segments() {
    let base_dir = "./fixtures/";
    let options = GlobMatchOptions::default();

    assert!(pattern_has_explicit_dot_for(
      "./fixtures/**/.*",
      base_dir,
      "./fixtures/.env",
      &options
    ));
    assert!(pattern_has_explicit_dot_for(
      "./fixtures/**/.*/index.js",
      base_dir,
      "./fixtures/.cache/index.js",
      &options
    ));
    assert!(!pattern_has_explicit_dot_for(
      "./fixtures/**/index.js",
      base_dir,
      "./fixtures/.cache/index.js",
      &options
    ));
  }

  #[test]
  fn explicit_dot_patterns_respect_case_insensitive_matching() {
    let base_dir = "./fixtures/";
    let options = GlobMatchOptions {
      case_sensitive: false,
      ..Default::default()
    };

    assert!(pattern_has_explicit_dot_for(
      "./fixtures/**/.ENV",
      base_dir,
      "./fixtures/.env",
      &options
    ));
  }

  #[test]
  fn glob_match_with_explicit_dot_requires_literal_dot_segments() {
    let options = GlobMatchOptions::default();
    assert!(glob_match_with_explicit_dot(
      "./fixtures/.*.js",
      "./fixtures/.hidden.js",
      "./fixtures/",
      &options
    ));
    assert!(!glob_match_with_explicit_dot(
      "./fixtures/*.js",
      "./fixtures/.hidden.js",
      "./fixtures/",
      &options
    ));
  }
}
