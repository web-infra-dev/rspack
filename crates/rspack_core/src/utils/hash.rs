use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rustc_hash::FxHashSet as HashSet;

pub fn calc_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

pub struct ExtractedHashPattern {
  pub pattern: String,
  pub len: Option<usize>,
}

/// Extract `[hash]` or `[hash:8]` in the template
pub fn extract_hash_pattern(pattern: &str, key: &str) -> Option<ExtractedHashPattern> {
  let key_offset = key.len() - 1;
  let start = pattern.find(&key[..key_offset])?;
  let end = pattern[start + key_offset..].find(']')?;
  let len = pattern[start + key_offset..start + key_offset + end]
    .strip_prefix(':')
    .and_then(|n| n.parse::<usize>().ok());

  let pattern = &pattern[start..=start + key_offset + end];
  Some(ExtractedHashPattern {
    pattern: pattern.to_string(),
    len,
  })
}

pub fn include_hash(filename: &str, hashes: &HashSet<String>) -> bool {
  hashes.iter().any(|hash| filename.contains(hash))
}

pub trait Replacer {
  fn get_replacer(&mut self, hash_len: Option<usize>) -> Cow<'_, str>;
}

impl Replacer for &str {
  #[inline]
  fn get_replacer(&mut self, _: Option<usize>) -> Cow<'_, str> {
    Cow::Borrowed(self)
  }
}

impl Replacer for &String {
  #[inline]
  fn get_replacer(&mut self, _: Option<usize>) -> Cow<'_, str> {
    Cow::Borrowed(self.as_str())
  }
}

impl<F, S> Replacer for F
where
  F: FnMut(Option<usize>) -> S,
  S: AsRef<str>,
{
  #[inline]
  fn get_replacer(&mut self, hash_len: Option<usize>) -> Cow<'_, str> {
    Cow::Owned((*self)(hash_len).as_ref().to_string())
  }
}

/// Replace all `[placeholder]` or `[placeholder:8]` in the pattern
pub fn replace_all_placeholder<'a>(
  pattern: &'a str,
  placeholder: &'a str,
  mut replacer: impl Replacer,
) -> Cow<'a, str> {
  let offset = placeholder.len() - 1;
  let mut iter = pattern.match_indices(&placeholder[..offset]).peekable();

  if iter.peek().is_none() {
    return Cow::Borrowed(pattern);
  }

  let mut ending = 0;
  let mut result = String::with_capacity(pattern.len());

  for (start, _) in iter {
    if start < ending {
      continue;
    }

    let start_offset = start + offset;
    if let Some(end) = pattern[start_offset..].find(']') {
      let end = start_offset + end;

      let replacer = replacer.get_replacer(
        pattern[start_offset..end]
          .strip_prefix(':')
          .and_then(|n| n.parse::<usize>().ok()),
      );

      result.push_str(&pattern[ending..start]);
      result.push_str(replacer.as_ref());

      ending = end + 1;
    }
  }

  if ending < pattern.len() {
    result.push_str(&pattern[ending..]);
  }

  Cow::Owned(result)
}

#[test]
fn test_replace_all_placeholder() {
  let result = replace_all_placeholder("hello-[hash].js", "[hash]", "abc");
  assert_eq!(result, "hello-abc.js");
  let result = replace_all_placeholder("hello-[hash]-[hash:5].js", "[hash]", |n: Option<usize>| {
    &"abcdefgh"[..n.unwrap_or(8)]
  });
  assert_eq!(result, "hello-abcdefgh-abcde.js");
}
