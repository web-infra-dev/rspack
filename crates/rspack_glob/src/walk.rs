use std::{borrow::Cow, sync::Arc};

use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::node_path::NodePath;

use crate::GlobPattern;

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
    walk(root, self, &fs, &mut files).await?;
    Ok(files)
  }
}

#[async_recursion]
async fn walk(
  root: &Utf8Path,
  pattern: &GlobPattern<'_>,
  fs: &Arc<dyn ReadableFileSystem>,
  files: &mut Vec<Utf8PathBuf>,
) -> Result<()> {
  if !fs.metadata(root).await.is_ok_and(|meta| meta.is_directory) {
    return Ok(());
  }
  let names = match pattern.view().next_components(&pattern.states) {
    Some(names) => names,
    None => fs.read_dir(root).await?,
  };
  for name in names {
    let Some(remaining) = pattern.match_prefix(&name) else {
      continue;
    };
    let path = root.join(&name);
    let Ok(metadata) = fs.metadata(&path).await else {
      continue;
    };
    if metadata.is_directory {
      if let Some(remaining) = remaining.match_prefix("/") {
        walk(&path, &remaining, fs, files).await?;
      }
    } else if remaining.match_path("").is_exact() {
      files.push(path);
    }
  }
  Ok(())
}
