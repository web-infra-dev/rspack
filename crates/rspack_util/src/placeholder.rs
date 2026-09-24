use std::{borrow::Cow, ops::Range};

use memchr::memmem;

/// A reusable scanner for a fixed, non-empty placeholder or placeholder prefix.
/// Store it in a static `LazyLock` to share the precomputed searcher across sources.
#[derive(Debug)]
pub struct PlaceholderFinder {
  finder: memmem::Finder<'static>,
}

impl PlaceholderFinder {
  pub fn new(marker: &'static str) -> Self {
    assert!(!marker.is_empty(), "placeholder marker must not be empty");
    Self {
      finder: memmem::Finder::new(marker),
    }
  }

  /// Finds non-overlapping placeholders and parses the text after the prefix.
  /// The parser returns the number of bytes consumed and a value, or `None` for
  /// an invalid placeholder. Consumed lengths must end on UTF-8 boundaries.
  /// Returned ranges include the prefix and use byte offsets for `ReplaceSource`.
  pub fn find_all<'a, T: 'a>(
    &'a self,
    source: &'a str,
    mut parse: impl FnMut(&'a str) -> Option<(usize, T)> + 'a,
  ) -> impl Iterator<Item = (Range<usize>, T)> + 'a {
    let mut offset = 0;
    std::iter::from_fn(move || {
      loop {
        let start = offset + self.finder.find(&source.as_bytes()[offset..])?;
        let value_start = start + self.finder.needle().len();
        if let Some((len, value)) = parse(&source[value_start..]) {
          offset = value_start + len;
          debug_assert!(source.is_char_boundary(offset));
          return Some((start..offset, value));
        }
        // A rejected prefix may overlap the next valid one. Search bytes so this
        // also works when advancing into a multi-byte character in the prefix.
        offset = start + 1;
      }
    })
  }

  /// Finds placeholders whose prefix is followed by a generated decimal ID.
  pub fn find_numeric<'a>(
    &'a self,
    source: &'a str,
  ) -> impl Iterator<Item = (Range<usize>, &'a str)> + 'a {
    self.find_all(source, |rest| {
      let len = rest.bytes().take_while(u8::is_ascii_digit).count();
      (len > 0).then(|| (len, &rest[..len]))
    })
  }

  /// Finds complete literal placeholders and returns their byte ranges.
  pub fn find_iter<'a>(&'a self, source: &'a str) -> impl Iterator<Item = Range<usize>> + 'a {
    self
      .finder
      .find_iter(source.as_bytes())
      .map(|start| start..start + self.finder.needle().len())
  }

  /// Replaces complete literal placeholders, borrowing the source when none match.
  /// Replacement text is inserted literally and is not scanned again.
  pub fn replace<'a>(&self, source: &'a str, replacement: &str) -> Cow<'a, str> {
    self.replace_with(source, || Cow::Borrowed(replacement))
  }

  /// Computes replacement text once, only if a placeholder is found, and resumes
  /// the same search to replace all matches without rescanning the first match.
  pub fn replace_with<'a, 'r>(
    &self,
    source: &'a str,
    replacement: impl FnOnce() -> Cow<'r, str>,
  ) -> Cow<'a, str> {
    let mut matches = self.find_iter(source).peekable();
    if matches.peek().is_none() {
      return Cow::Borrowed(source);
    }

    let replacement = replacement();
    let mut result = String::with_capacity(source.len());
    let mut end = 0;
    for range in matches {
      result.push_str(&source[end..range.start]);
      result.push_str(&replacement);
      end = range.end;
    }
    result.push_str(&source[end..]);
    Cow::Owned(result)
  }
}
