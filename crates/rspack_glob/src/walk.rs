use std::{borrow::Cow, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::node_path::NodePath;

use crate::GlobPattern;

/// Walk a directory tree depth-first, calling `on_file` for each file found.
///
/// - `root`: starting directory
/// - `recursive`: whether to descend into subdirectories
/// - `skip_dotfiles`: whether to skip files whose name starts with `.`
/// - `should_enter_dir`: called with (full_path, dirname) for each directory before descending
/// - `on_file`: called with (full_path, filename) for each file
pub async fn walk_dir(
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
  let filenames = fs.read_dir(root).await?;
  // Keep each directory's iterator suspended while visiting its children, so
  // callbacks retain the depth-first order without recursive boxed futures.
  let mut stack = vec![(root.to_path_buf(), filenames.into_iter())];
  while let Some((root, filenames)) = stack.last_mut() {
    let Some(filename) = filenames.next() else {
      stack.pop();
      continue;
    };
    let path = root.join(&filename);
    if fs.metadata(&path).await.is_ok_and(|m| m.is_directory) {
      if recursive && should_enter_dir(&path, &filename) {
        let filenames = fs.read_dir(&path).await?;
        stack.push((path, filenames.into_iter()));
      }
    } else if skip_dotfiles && filename.starts_with('.') {
      // skip dotfiles
    } else {
      on_file(path, filename);
    }
  }
  Ok(())
}

impl GlobPattern<'_> {
  fn scan_directory_prefix(&self) -> Cow<'_, str> {
    // A directory boundary is also a UTF-8 boundary for filesystem paths.
    let prefix = String::from_utf8_lossy(self.directory_prefix());
    if self.options.case_sensitive {
      return prefix;
    }
    // Case folding must not change the spelling used for direct filesystem
    // lookup. Only structural path components can be consumed without readdir.
    let end = prefix
      .split_inclusive('/')
      .enumerate()
      .take_while(|(index, segment)| {
        matches!(segment.trim_end_matches('/'), "" | "." | "..")
          || (*index == 0 && segment.ends_with(":/"))
      })
      .map(|(_, segment)| segment.len())
      .sum();
    match prefix {
      Cow::Borrowed(prefix) => Cow::Borrowed(&prefix[..end]),
      Cow::Owned(mut prefix) => {
        prefix.truncate(end);
        Cow::Owned(prefix)
      }
    }
  }

  /// The stable directory containing every match, for filesystem watching.
  /// `context` is a literal physical path and may contain glob metacharacters.
  pub fn scan_root(&self, context: &Utf8Path) -> Utf8PathBuf {
    let prefix = self.scan_directory_prefix();
    if Utf8Path::new(&prefix).is_absolute() {
      Utf8Path::new(&prefix).node_normalize()
    } else {
      context.node_join(prefix.as_ref()).node_normalize()
    }
  }

  /// Find matching files relative to a literal context, or from an absolute glob.
  /// Consumes the common literal directory prefix before traversing the rest.
  pub async fn scan(
    &self,
    context: &Utf8Path,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Vec<Utf8PathBuf>> {
    let root = self.scan_root(context);
    let Some(remaining) = self.match_prefix(self.scan_directory_prefix().as_bytes()) else {
      return Ok(Vec::new());
    };
    remaining.walk(&root, fs).await
  }

  /// Find files under a physical directory using this pattern's remaining states.
  /// No part of the physical root is interpreted as glob syntax.
  pub async fn walk(
    &self,
    root: &Utf8Path,
    fs: Arc<dyn ReadableFileSystem>,
  ) -> Result<Vec<Utf8PathBuf>> {
    let mut files = Vec::new();
    if !fs.metadata(root).await.is_ok_and(|meta| meta.is_directory) {
      return Ok(files);
    }
    let names = match self.view().next_components(&self.states) {
      Some(names) => names,
      None => fs.read_dir(root).await?,
    };
    // Suspend parent iterators and matching states to preserve depth-first
    // traversal while pruning children with the remaining pattern.
    let mut stack = vec![(root.to_path_buf(), Cow::Borrowed(self), names.into_iter())];
    while let Some((root, pattern, names)) = stack.last_mut() {
      let Some(name) = names.next() else {
        stack.pop();
        continue;
      };
      let Some(remaining) = pattern.match_prefix(&name) else {
        continue;
      };
      let path = root.join(&name);
      let Ok(metadata) = fs.metadata(&path).await else {
        continue;
      };
      if metadata.is_directory {
        if let Some(remaining) = remaining.match_prefix("/") {
          let names = match remaining.view().next_components(&remaining.states) {
            Some(names) => names,
            None => fs.read_dir(&path).await?,
          };
          stack.push((path, Cow::Owned(remaining), names.into_iter()));
        }
      } else if remaining.match_path("").is_exact() {
        files.push(path);
      }
    }
    Ok(files)
  }
}
