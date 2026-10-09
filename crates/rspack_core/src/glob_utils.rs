use core::fmt;
use std::sync::Arc;

use async_recursion::async_recursion;
use cow_utils::CowUtils;
use rspack_error::{Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_glob::{GlobOptions, GlobPattern};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::node_path::NodePath;

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

impl From<&GlobMatchOptions> for GlobOptions {
  fn from(options: &GlobMatchOptions) -> Self {
    Self {
      case_sensitive: options.case_sensitive,
      require_literal_leading_dot: options.require_literal_leading_dot,
    }
  }
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
  s.cow_replace('\\', "/").into_owned()
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

/// Walk a directory tree recursively, calling `on_file` for each file found.
///
/// - `root`: starting directory
/// - `recursive`: whether to descend into subdirectories
/// - `skip_dotfiles`: whether to skip files whose name starts with `.`
/// - `should_enter_dir`: called with (full_path, dirname) for each directory before descending
/// - `on_file`: called with (full_path, filename) for each file
#[async_recursion]
pub(crate) async fn walk_dir(
  root: &Utf8Path,
  fs: Arc<dyn ReadableFileSystem>,
  recursive: bool,
  skip_dotfiles: bool,
  should_enter_dir: &mut (impl FnMut(&Utf8Path, &str) -> bool + Send),
  on_file: &mut (impl FnMut(Utf8PathBuf, String) + Send),
) -> Result<()> {
  if !fs.metadata(root).await.is_ok_and(|m| m.is_directory) {
    return Ok(());
  }
  for filename in fs.read_dir(root).await? {
    let path = root.join(&filename);
    if fs.metadata(&path).await.is_ok_and(|m| m.is_directory) {
      if recursive && should_enter_dir(&path, &filename) {
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

/// A physical scan root and a compiled glob in coordinates relative to that root.
/// Literal filesystem paths never become part of the glob's syntax.
#[derive(Debug)]
pub struct GlobScanPlan {
  pub root: Utf8PathBuf,
  pub pattern: GlobPattern,
  literal_path: Option<Utf8PathBuf>,
}

impl GlobScanPlan {
  /// Compile a glob relative to `context`, consuming its literal directory prefix.
  pub fn new(pattern: &str, context: &Utf8Path, options: &GlobMatchOptions) -> Result<Self> {
    let normalized = normalize_path_separators(pattern);
    let end = if options.case_sensitive {
      glob_base_dir_end(&normalized)
    } else {
      // Keep the real spelling of directory entries when matching case-insensitively.
      normalized
        .split_inclusive('/')
        .enumerate()
        .take_while(|(index, segment)| {
          matches!(segment.trim_end_matches('/'), "" | "." | "..")
            || (*index == 0 && segment.ends_with(":/"))
        })
        .map(|(_, segment)| segment.len())
        .sum()
    };
    let prefix = unescape_glob_path(&normalized[..end]);
    let join_literal = |path: &str| {
      if Utf8Path::new(path).is_absolute() {
        Utf8Path::new(path).node_normalize_posix()
      } else {
        context.node_join_posix(path).node_normalize_posix()
      }
    };
    let root = join_literal(&prefix);
    let compiled = GlobPattern::new_with_options(&normalized, options.into())
      .map_err(|err| error!("Invalid glob pattern {pattern:?}: {err}"))?;
    let remaining = compiled
      .match_prefix(&prefix)
      .ok_or_else(|| error!("Glob pattern {pattern:?} does not match its directory prefix"))?;
    let literal_path = (options.case_sensitive
      && first_glob_metacharacter_index(&normalized).is_none())
    .then(|| join_literal(&unescape_glob_path(&normalized)));
    Ok(Self {
      root,
      pattern: remaining,
      literal_path,
    })
  }

  /// Scan a directory with a pattern that is already relative to it.
  pub fn from_directory(root: Utf8PathBuf, pattern: GlobPattern) -> Self {
    Self {
      root,
      pattern,
      literal_path: None,
    }
  }

  /// Traverse only directories for which the pattern has a continuation.
  pub async fn scan(&self, fs: Arc<dyn ReadableFileSystem>) -> Result<Vec<Utf8PathBuf>> {
    if let Some(path) = &self.literal_path {
      return Ok(
        if fs.metadata(path).await.is_ok_and(|meta| !meta.is_directory)
          && self.pattern.is_match(path.file_name().unwrap_or_default())
        {
          vec![path.clone()]
        } else {
          Vec::new()
        },
      );
    }
    let mut results = Vec::new();
    scan_glob_dir(&self.root, &self.pattern, &fs, &mut results).await?;
    Ok(results)
  }
}

#[async_recursion]
async fn scan_glob_dir(
  root: &Utf8Path,
  pattern: &GlobPattern,
  fs: &Arc<dyn ReadableFileSystem>,
  results: &mut Vec<Utf8PathBuf>,
) -> Result<()> {
  if !fs.metadata(root).await.is_ok_and(|meta| meta.is_directory) {
    return Ok(());
  }
  for filename in fs.read_dir(root).await? {
    let Some(remaining) = pattern.match_prefix(&filename) else {
      continue;
    };
    let path = root.join(&filename);
    if fs.metadata(&path).await.is_ok_and(|meta| meta.is_directory) {
      if let Some(remaining) = remaining.match_prefix("/") {
        scan_glob_dir(&path, &remaining, fs, results).await?;
      }
    } else if remaining.is_match("") {
      results.push(path);
    }
  }
  Ok(())
}

/// Find files matching a glob by advancing its states during directory traversal.
pub async fn find_files_by_glob(
  pattern: &str,
  options: &GlobMatchOptions,
  fs: Arc<dyn ReadableFileSystem>,
) -> Result<Vec<Utf8PathBuf>> {
  GlobScanPlan::new(pattern, Utf8Path::new("."), options)?
    .scan(fs)
    .await
}
