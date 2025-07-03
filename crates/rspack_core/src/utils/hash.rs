use std::{
  borrow::Cow,
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

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
  fn get(&mut self, dst: &mut String, hash_len: Option<usize>, need_base64: bool);
}

impl Replacer for &str {
  #[inline]
  fn get(&mut self, dst: &mut String, _: Option<usize>, _: bool) {
    dst.push_str(self);
  }
}

impl Replacer for &String {
  #[inline]
  fn get(&mut self, dst: &mut String, _: Option<usize>, _: bool) {
    dst.push_str(self);
  }
}

impl<F, S> Replacer for F
where
  F: FnMut(Option<usize>, bool) -> S,
  S: AsRef<str>,
{
  #[inline]
  fn get(&mut self, dst: &mut String, hash_len: Option<usize>, need_base64: bool) {
    dst.push_str((*self)(hash_len, need_base64).as_ref())
  }
}

fn replace_all_placeholder_impl<'a>(
  pattern: &'a str,
  with_extra: bool,
  mut placeholder: &'a str,
  mut replacer: impl Replacer,
) -> Cow<'a, str> {
  let offset = placeholder.len() - 1;

  if with_extra {
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
    let (end, len, need_base64) = if with_extra {
      let rest = &pattern[start_offset..];
      let Some(end) = rest.find(']') else {
        continue;
      };
      if end == 0 {
        (start_offset, None, false)
      } else {
        let matched = &rest[1..end];
        let mut configs = matched.rsplit(':');
        let len = if let Some(len) = configs.next() {
          match len.parse::<usize>() {
            Ok(len) => Some(len),
            Err(_) => continue,
          }
        } else {
          None
        };
        let need_base64 = if let Some(base64) = configs.next() {
          if base64 == "base64" {
            true
          } else {
            continue;
          }
        } else {
          false
        };
        (start_offset + end, len, need_base64)
      }
    } else {
      (start_offset, None, false)
    };

    result.push_str(&pattern[last_end..start]);
    replacer.get(&mut result, len, need_base64);

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

  let result = "hello-[hash]-[hash:5]-[hash_name]-[hash:o].js".replace_all_with_len(
    "[hash]",
    |len: Option<usize>, base64: bool| {
      assert!(!base64);
      &"abcdefgh"[..len.unwrap_or(8)]
    },
  );
  assert_eq!(result, "hello-abcdefgh-abcde-[hash_name]-[hash:o].js");

  let result =
    "[hash:base64:4]".replace_all_with_len("[hash]", |len: Option<usize>, base64: bool| {
      assert!(base64);
      &"abcdefgh"[..len.unwrap_or(8)]
    });
  assert_eq!(result, "abcd");
}
