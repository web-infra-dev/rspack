use std::borrow::Cow;

use rspack_paths::{Utf8Path, Utf8PathBuf};

use super::GlobPattern;

/// The directory spelled out before the first glob construct in a pattern with
/// normalized separators. Escaped metacharacters are literal path characters.
pub fn literal_directory_prefix(pattern: &str) -> Cow<'_, str> {
  let mut bytes = pattern.bytes().enumerate();
  let mut end = 0;
  while let Some((index, byte)) = bytes.next() {
    match byte {
      b'*' | b'?' | b'[' | b'{' => break,
      b'/' => end = index + 1,
      b'\\' => {
        bytes.next();
      }
      _ => {}
    }
  }

  let prefix = &pattern[..end];
  if !prefix.contains('\\') {
    return Cow::Borrowed(prefix);
  }
  let mut chars = prefix.chars();
  Cow::Owned(
    std::iter::from_fn(|| match chars.next()? {
      '\\' => chars.next(),
      ch => Some(ch),
    })
    .collect(),
  )
}

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
          || (*index == 0 && Utf8Path::new(segment).is_absolute())
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

  /// The starting directory for matching against physical filesystem entries.
  /// `context` is a literal physical path and may contain glob metacharacters.
  pub fn scan_root(&self, context: &Utf8Path) -> Utf8PathBuf {
    context.join(self.scan_directory_prefix().as_ref())
  }
}
