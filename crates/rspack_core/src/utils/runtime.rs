use std::borrow::Cow;

use indexmap::IndexMap;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{merge_runtime, EntryData, EntryOptions, Filename, RuntimeSpec};

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
  let len = if let Some(n) = pattern[start + key_offset..start + key_offset + end].strip_prefix(':')
  {
    Some(n.parse::<usize>().ok()?)
  } else {
    None
  };
  let pattern = &pattern[start..=start + key_offset + end];
  Some(ExtractedHashPattern {
    pattern: pattern.to_string(),
    len,
  })
}

pub fn replace_all_hash_pattern<'a>(pattern: &'a str, key: &'a str, hash: &'a str) -> Cow<'a, str> {
  let key_offset = key.len() - 1;
  let key = &key[..key_offset];

  let Some(start) = pattern.find(key) else {
    return Cow::Borrowed(pattern);
  };

  let Some(end) = pattern[start + key_offset..].find(']') else {
    return Cow::Borrowed(pattern);
  };

  let hash_len = hash.len();
  let mut result = String::with_capacity(pattern.len());

  result.push_str(&pattern[..start]);
  result.push_str(
    &hash[..pattern[start + key_offset..start + key_offset + end]
      .strip_prefix(':')
      .map(|n| n.parse::<usize>().ok().unwrap_or(hash_len).min(hash_len))
      .unwrap_or(hash_len)],
  );

  let mut offset = start + key_offset + end;

  while let Some(start) = pattern[offset..].find(key) {
    let start = offset + start;
    if let Some(end) = pattern[start + key_offset..].find(']') {
      let hash_len = pattern[start + key_offset..start + key_offset + end]
        .strip_prefix(':')
        .map(|n| n.parse::<usize>().ok().unwrap_or(hash_len).min(hash_len))
        .unwrap_or(hash_len);

      result.push_str(&pattern[offset + 1..start]);
      result.push_str(&hash[..hash_len]);

      offset = start + key_offset + end;
    } else {
      result.push_str(&pattern[offset + 1..]);
      return Cow::Owned(result);
    }
  }

  result.push_str(&pattern[offset + 1..]);

  Cow::Owned(result)
}

#[test]
fn test_replace_all_hash_pattern() {
  let result = replace_all_hash_pattern("hello-[hash].js", "[hash]", "abc");
  assert_eq!(result.as_ref(), "hello-abc.js");
  let result = replace_all_hash_pattern("hello-[hash]-[hash:5].js", "[hash]", "abcdefgh");
  assert_eq!(result.as_ref(), "hello-abcdefgh-abcde.js");
}

pub fn get_filename_without_hash_length<F: Clone>(
  filename: &Filename<F>,
) -> (Filename<F>, HashMap<String, usize>) {
  let mut hash_len_map = HashMap::default();
  let Some(template) = filename.template() else {
    return (filename.clone(), hash_len_map);
  };
  let mut template = template.to_string();
  for key in ["[hash]", "[fullhash]", "[chunkhash]", "[contenthash]"] {
    if let Some(p) = extract_hash_pattern(&template, key) {
      if let Some(hash_len) = p.len {
        hash_len_map.insert((*key).to_string(), hash_len);
      }
      template = template.replace(&p.pattern, key);
    }
  }
  (Filename::from(template), hash_len_map)
}
