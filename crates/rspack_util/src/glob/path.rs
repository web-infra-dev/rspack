use std::borrow::Cow;

use rspack_fs::WalkMatcher;
use rspack_paths::{Utf8Path, Utf8PathBuf};

use super::GlobPattern;
use crate::node_path::NodePath;

impl GlobPattern<'_> {
  /// Literal directory prefix that can be used for a filesystem lookup.
  pub fn scan_directory_prefix(&self) -> Cow<'_, str> {
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
}

impl WalkMatcher for GlobPattern<'_> {
  fn literal_children(&self) -> Option<Vec<String>> {
    self.view().next_components(&self.states)
  }

  fn prepare_child(&self, _dir: &Utf8Path, name: &str) -> Option<Self> {
    self.match_prefix(name)
  }

  fn match_entry(self, _dir: &Utf8Path, _name: &str, is_directory: bool) -> Option<Self> {
    if is_directory {
      self.match_prefix("/")
    } else {
      self.match_path("").is_exact().then_some(self)
    }
  }
}
