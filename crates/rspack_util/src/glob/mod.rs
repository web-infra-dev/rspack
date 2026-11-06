//! Shared glob matching and filesystem traversal.

mod dfa;

use core::fmt;
use std::sync::Arc;

use cow_utils::CowUtils;
pub use dfa::{GlobDfa, GlobDfaState, GlobExpression};
use fast_glob::glob_match;
use rspack_error::Result;
use rspack_fs::{ReadableFileSystem, WalkMatcher};
use rspack_paths::{Utf8Path, Utf8PathBuf};

#[derive(Debug)]
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
  let normalized_path = normalize_path_separators_for_path(path);
  let normalized_base_dir = normalize_path_separators_for_path(base_dir);

  glob_match_normalized_with_explicit_dot(
    &normalized_pattern,
    &normalized_path,
    &normalized_base_dir,
    options,
  )
}

/// Match normalized path strings against a normalized glob pattern.
#[doc(hidden)]
pub fn glob_match_normalized_with_explicit_dot(
  normalized_pattern: &str,
  normalized_path: &str,
  normalized_base_dir: &str,
  options: &GlobMatchOptions,
) -> bool {
  glob_allows_dot_components(
    normalized_pattern,
    normalized_path,
    normalized_base_dir,
    options,
  ) && glob_match_with_options(normalized_pattern, normalized_path, options)
}

/// Check the dot-component policy after the compiled matcher accepts a path.
#[doc(hidden)]
pub fn glob_allows_dot_components(
  normalized_pattern: &str,
  normalized_path: &str,
  normalized_base_dir: &str,
  options: &GlobMatchOptions,
) -> bool {
  !options.require_literal_leading_dot
    || !path_has_dot_component(normalized_path, normalized_base_dir)
    || pattern_has_explicit_dot_for(
      normalized_pattern,
      normalized_base_dir,
      normalized_path,
      options,
    )
}

/// Return whether a character has special meaning in glob patterns.
pub fn is_glob_metacharacter(c: char) -> bool {
  matches!(c, '*' | '?' | '[' | '{')
}

/// Return the byte index of the first unescaped glob metacharacter in a pattern.
fn first_glob_metacharacter_index(pattern: &str) -> Option<usize> {
  let mut escaped = false;
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
      return Some(byte_idx);
    }
  }

  None
}

/// Return the byte index after the base directory prefix of a glob pattern.
pub fn glob_base_dir_end(pattern: &str) -> usize {
  let idx = first_glob_metacharacter_index(pattern).unwrap_or(pattern.len());

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

/// Normalize backslashes to forward slashes in a literal filesystem path.
pub fn normalize_path_separators_for_path(s: &str) -> String {
  rspack_paths::normalize_path_separators(s).into_owned()
}

pub fn unescape_glob_path(s: &str) -> String {
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

#[derive(Clone, Copy)]
struct GlobWalkMatcher<'a> {
  pattern: &'a str,
  base_dir: &'a str,
  options: &'a GlobMatchOptions,
  cursor: Option<GlobDfaState<'a>>,
}

impl WalkMatcher for GlobWalkMatcher<'_> {
  const VISIT_ON_METADATA_ERROR: bool = true;

  fn prepare_child(&self, _dir: &Utf8Path, name: &str) -> Option<Self> {
    let mut child = *self;
    if let Some(cursor) = child.cursor {
      let name = rspack_paths::normalize_path_separators(name);
      let next = if child.options.case_sensitive {
        cursor.advance(&name)
      } else {
        cursor.advance(&name.cow_to_lowercase())
      };
      if !next.can_include() {
        return None;
      }
      child.cursor = Some(next);
    }
    Some(child)
  }

  fn match_entry(mut self, dir: &Utf8Path, name: &str, is_directory: bool) -> Option<Self> {
    if is_directory {
      if let Some(cursor) = self.cursor {
        let next = cursor.advance("/");
        if !next.can_include() {
          return None;
        }
        self.cursor = Some(next);
      }
      return Some(self);
    }
    let path = dir.join(name);
    let path = rspack_paths::normalize_path_separators(path.as_str());
    let matched = if let Some(cursor) = self.cursor {
      cursor.matches().next().is_some()
        && glob_allows_dot_components(self.pattern, &path, self.base_dir, self.options)
    } else {
      glob_match_normalized_with_explicit_dot(self.pattern, &path, self.base_dir, self.options)
    };
    matched.then_some(self)
  }
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

  // A pattern without metacharacters can only match itself, so walking its base directory is wasted
  // work. Case-insensitive matching still needs the walk to find the real spelling on disk.
  if options.case_sensitive && first_glob_metacharacter_index(&normalized_pattern).is_none() {
    let path = Utf8PathBuf::from(unescape_glob_path(&normalized_pattern));
    let matched = fs
      .metadata(&path)
      .await
      .is_ok_and(|meta| !meta.is_directory)
      && glob_match_normalized_with_explicit_dot(
        &normalized_pattern,
        path.as_str(),
        base_dir_path.as_str(),
        options,
      );

    return Ok(if matched { vec![path] } else { Vec::new() });
  }

  let program = {
    let pattern = if options.case_sensitive {
      std::borrow::Cow::Borrowed(normalized_pattern.as_str())
    } else {
      normalized_pattern.cow_to_lowercase()
    };
    GlobExpression::new("", &pattern).and_then(|pattern| GlobDfa::new(&[pattern], 1))
  };
  let cursor = program.as_ref().map(|program| {
    let directory = if options.case_sensitive {
      std::borrow::Cow::Borrowed(base_dir_path.as_str())
    } else {
      base_dir_path.as_str().cow_to_lowercase()
    };
    let state = program.start().advance(&directory);
    if directory.ends_with('/') {
      state
    } else {
      state.advance("/")
    }
  });
  let matcher = GlobWalkMatcher {
    pattern: &normalized_pattern,
    base_dir: base_dir_path.as_str(),
    options,
    cursor,
  };
  let mut results = Vec::new();
  matcher
    .walk(base_dir_path, fs.as_ref(), false, &mut |path, _| {
      results.push(path)
    })
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
#[doc(hidden)]
pub fn pattern_has_explicit_dot_for(
  pattern: &str,
  base_dir: &str,
  path: &str,
  options: &GlobMatchOptions,
) -> bool {
  let escaped_base_dir = escape_glob_pattern(base_dir);
  let pattern_suffix = pattern
    .strip_prefix(base_dir)
    .or_else(|| pattern.strip_prefix(&escaped_base_dir))
    .unwrap_or(pattern);

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
