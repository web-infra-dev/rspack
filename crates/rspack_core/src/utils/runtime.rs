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
  let start = pattern.find(&key[..key.len() - 1])?;
  let end = pattern[start + 5..].find(']')?;
  let len = if let Some(n) = pattern[start + 5..start + 5 + end].strip_prefix(':') {
    Some(n.parse::<usize>().ok()?)
  } else {
    None
  };
  let pattern = &pattern[start..=start + 5 + end];
  Some(ExtractedHashPattern {
    pattern: pattern.to_string(),
    len,
  })
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
