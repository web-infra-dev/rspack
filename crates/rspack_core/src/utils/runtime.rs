use std::borrow::Cow;

use cow_utils::CowUtils;
use indexmap::IndexMap;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{merge_runtime, EntryData, EntryOptions, Filename, RuntimeSpec};
use crate::{
  CHUNK_HASH_PLACEHOLDER, CONTENT_HASH_PLACEHOLDER, FULL_HASH_PLACEHOLDER, HASH_PLACEHOLDER,
};

pub fn get_entry_runtime(
  name: &str,
  options: &EntryOptions,
  entries: &IndexMap<String, EntryData>,
) -> RuntimeSpec {
  if let Some(depend_on) = &options.depend_on {
    let mut result: RuntimeSpec = Default::default();
    let mut queue = vec![];
    queue.extend(depend_on.clone());

    let mut visited = HashSet::<String>::default();

    while let Some(name) = queue.pop() {
      if visited.contains(&name) {
        continue;
      }
      visited.insert(name.clone());
      let Some(EntryData { options, .. }) = entries.get(&name) else {
        continue;
      };

      if let Some(depend_on) = &options.depend_on {
        for depend in depend_on {
          queue.push(depend.clone());
        }
      } else {
        result = merge_runtime(
          &result,
          &RuntimeSpec::from_entry(&name, options.runtime.as_ref()),
        );
      }
    }
    result
  } else {
    RuntimeSpec::from_entry(name, options.runtime.as_ref())
  }
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

/// Replace all `[hash]` or `[hash:8]` in the pattern
pub fn replace_all_hash_pattern<'a, F, S>(
  pattern: &'a str,
  key: &'a str,
  mut hash: F,
) -> Option<String>
where
  F: FnMut(Option<usize>) -> S,
  S: AsRef<str>,
{
  let offset = key.len() - 1;
  let mut iter = pattern.match_indices(&key[..offset]).peekable();

  iter.peek()?;

  let mut ending = 0;
  let mut result = String::with_capacity(pattern.len());

  for (start, _) in iter {
    if start < ending {
      continue;
    }

    let start_offset = start + offset;
    if let Some(end) = pattern[start_offset..].find(']') {
      let end = start_offset + end;

      let hash = hash(
        pattern[start_offset..end]
          .strip_prefix(':')
          .and_then(|n| n.parse::<usize>().ok()),
      );

      result.push_str(&pattern[ending..start]);
      result.push_str(hash.as_ref());

      ending = end + 1;
    }
  }

  if ending < pattern.len() {
    result.push_str(&pattern[ending..]);
  }

  Some(result)
}

#[test]
fn test_replace_all_hash_pattern() {
  let result = replace_all_hash_pattern("hello-[hash].js", "[hash]", |_| "abc");
  assert_eq!(result, Some("hello-abc.js".to_string()));
  let result = replace_all_hash_pattern("hello-[hash]-[hash:5].js", "[hash]", |n| {
    &"abcdefgh"[..n.unwrap_or(8)]
  });
  assert_eq!(result, Some("hello-abcdefgh-abcde.js".to_string()));
}

pub fn get_filename_without_hash_length<F: Clone>(
  filename: &Filename<F>,
) -> (Filename<F>, HashMap<String, usize>) {
  let mut hash_len_map = HashMap::default();
  let Some(template) = filename.template() else {
    return (filename.clone(), hash_len_map);
  };
  let mut template = Cow::Borrowed(template);
  for key in [
    HASH_PLACEHOLDER,
    FULL_HASH_PLACEHOLDER,
    CHUNK_HASH_PLACEHOLDER,
    CONTENT_HASH_PLACEHOLDER,
  ] {
    if let Some(p) = extract_hash_pattern(&template, key) {
      if let Some(hash_len) = p.len {
        hash_len_map.insert((*key).to_string(), hash_len);
      }
      template = Cow::Owned(template.cow_replace(&p.pattern, key).into_owned());
    }
  }
  (Filename::from(template.into_owned()), hash_len_map)
}
