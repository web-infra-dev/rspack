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
  fn get(&mut self, hash_len: Option<usize>) -> Cow<'_, str>;
}

impl Replacer for &str {
  #[inline]
  fn get(&mut self, _: Option<usize>) -> Cow<'_, str> {
    Cow::Borrowed(self)
  }
}

impl Replacer for &String {
  #[inline]
  fn get(&mut self, _: Option<usize>) -> Cow<'_, str> {
    Cow::Borrowed(self.as_str())
  }
}

impl<F, S> Replacer for F
where
  F: FnMut(Option<usize>) -> S,
  S: AsRef<str>,
{
  #[inline]
  fn get(&mut self, hash_len: Option<usize>) -> Cow<'_, str> {
    Cow::Owned((*self)(hash_len).as_ref().to_string())
  }
}

fn replace_all_placeholder_impl<'a>(
  pattern: &'a str,
  is_len_enabled: bool,
  mut placeholder: &'a str,
  mut replacer: impl Replacer,
) -> Cow<'a, str> {
  let offset = placeholder.len() - 1;

  if is_len_enabled {
    placeholder = &placeholder[..offset];
  }

  let mut iter = pattern.match_indices(placeholder).peekable();

  if iter.peek().is_none() {
    return Cow::Borrowed(pattern);
  }

  let mut last_end = 0;
  let mut result = String::with_capacity(pattern.len());

  for (start, _) in iter {
    if start < last_end {
      continue;
    }

    let start_offset = start + offset;
    let (end, len) = if is_len_enabled {
      let rest = &pattern[start_offset..];
      match rest.as_bytes().first() {
        Some(&b':') => {
          if let Some(index) = rest.find(']') {
            match rest[1..index].parse::<usize>() {
              Ok(len) => (start_offset + index, Some(len)),
              Err(_) => continue,
            }
          } else {
            continue;
          }
        }
        Some(&b']') => (start_offset, None),
        _ => continue,
      }
    } else {
      (start_offset, None)
    };

    let replacer = replacer.get(len);

    result.push_str(&pattern[last_end..start]);
    result.push_str(replacer.as_ref());

    last_end = end + 1;
  }

  if last_end < pattern.len() {
    result.push_str(&pattern[last_end..]);
  }

  Cow::Owned(result)
}

/// Replace all `[placeholder]` or `[placeholder:8]` in the pattern
pub trait ReplaceAllPlaceholder {
  fn replace_all<'a>(&'a self, placeholder: &'a str, replacer: impl Replacer) -> Cow<'a, str>;

  fn replace_all_with_len<'a>(
    &'a self,
    placeholder: &'a str,
    replacer: impl Replacer,
  ) -> Cow<'a, str>;
}

impl ReplaceAllPlaceholder for str {
  #[inline]
  fn replace_all<'a>(&'a self, placeholder: &'a str, replacer: impl Replacer) -> Cow<'a, str> {
    replace_all_placeholder_impl(self, false, placeholder, replacer)
  }

  #[inline]
  fn replace_all_with_len<'a>(
    &'a self,
    placeholder: &'a str,
    replacer: impl Replacer,
  ) -> Cow<'a, str> {
    replace_all_placeholder_impl(self, true, placeholder, replacer)
  }
}

#[test]
fn test_replace_all_placeholder() {
  let result = "hello-[hash]-[hash_name]-[hash:1].js".replace_all("[hash]", "abc");
  assert_eq!(result, "hello-abc-[hash_name]-[hash:1].js");

  let result =
    "hello-[hash]-[hash:-]-[hash_name]-[hash:1]-[hash:].js".replace_all_with_len("[hash]", "abc");
  assert_eq!(result, "hello-abc-[hash:-]-[hash_name]-abc-[hash:].js");

  let result = "hello-[hash]-[hash:5]-[hash_name]-[hash:o].js"
    .replace_all_with_len("[hash]", |n: Option<usize>| &"abcdefgh"[..n.unwrap_or(8)]);
  assert_eq!(result, "hello-abcdefgh-abcde-[hash_name]-[hash:o].js");
}
