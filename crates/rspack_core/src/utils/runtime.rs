use std::borrow::Cow;

use cow_utils::CowUtils;
use indexmap::IndexMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::extract_hash_pattern;
use crate::{
  CHUNK_HASH_PLACEHOLDER, CONTENT_HASH_PLACEHOLDER, EntryData, EntryOptions, FULL_HASH_PLACEHOLDER,
  Filename, HASH_PLACEHOLDER, RuntimeSpec,
};

pub fn get_entry_runtime(
  name: &str,
  options: &EntryOptions,
  entries: &IndexMap<String, EntryData>,
) -> RuntimeSpec {
  if let Some(depend_on) = &options.depend_on {
    let mut result: RuntimeSpec = Default::default();
    let mut queue = vec![];
    queue.extend(depend_on.iter().cloned());

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
        result.extend(&RuntimeSpec::from_entry(&name, options.runtime.as_ref()));
      }
    }
    result
  } else {
    RuntimeSpec::from_entry(name, options.runtime.as_ref())
  }
}

pub fn get_filename_without_hash_length(filename: &Filename) -> (Filename, HashMap<String, usize>) {
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
