use std::{borrow::Cow, ops::Range};

use memchr::memmem;

/// Finds non-overlapping placeholders with a fixed, non-empty prefix.
///
/// `parse` receives the text immediately after the prefix and returns the number
/// of bytes consumed and the parsed value, or `None` for an invalid placeholder.
/// The consumed length must end on a UTF-8 boundary. Returned ranges include the
/// prefix and use byte offsets, suitable for `ReplaceSource`.
pub fn find_placeholders<'a, T: 'a>(
  source: &'a str,
  prefix: &'a str,
  mut parse: impl FnMut(&'a str) -> Option<(usize, T)> + 'a,
) -> impl Iterator<Item = (Range<usize>, T)> + 'a {
  assert!(!prefix.is_empty(), "placeholder prefix must not be empty");
  let finder = memmem::Finder::new(prefix);
  let mut offset = 0;
  std::iter::from_fn(move || {
    loop {
      let start = offset + finder.find(&source.as_bytes()[offset..])?;
      let value_start = start + prefix.len();
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
pub fn find_numeric_placeholders<'a>(
  source: &'a str,
  prefix: &'a str,
) -> impl Iterator<Item = (Range<usize>, &'a str)> + 'a {
  find_placeholders(source, prefix, |rest| {
    let len = rest.bytes().take_while(u8::is_ascii_digit).count();
    (len > 0).then(|| (len, &rest[..len]))
  })
}

/// Finds complete literal placeholders and returns their byte ranges.
pub fn find_literal_placeholders<'a>(
  source: &'a str,
  placeholder: &'a str,
) -> impl Iterator<Item = Range<usize>> + 'a {
  find_placeholders(source, placeholder, |_| Some((0, ()))).map(|(range, ())| range)
}

/// Replaces complete literal placeholders, borrowing the source when none match.
/// Replacement text is inserted literally and is not scanned again.
pub fn replace_placeholder<'a>(
  source: &'a str,
  placeholder: &str,
  replacement: &str,
) -> Cow<'a, str> {
  let mut matches = find_literal_placeholders(source, placeholder).peekable();
  if matches.peek().is_none() {
    return Cow::Borrowed(source);
  }

  let mut result = String::with_capacity(source.len());
  let mut end = 0;
  for range in matches {
    result.push_str(&source[end..range.start]);
    result.push_str(replacement);
    end = range.end;
  }
  result.push_str(&source[end..]);
  Cow::Owned(result)
}
