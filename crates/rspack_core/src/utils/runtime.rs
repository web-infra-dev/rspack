use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use crate::{
  merge_runtime, Compilation, EntryOptions, Filename, RuntimeSpec, CHUNK_HASH_PLACEHOLDER,
  CONTENT_HASH_PLACEHOLDER, FULL_HASH_PLACEHOLDER, HASH_PLACEHOLDER,
};

pub fn get_entry_runtime(
  name: &str,
  options: &EntryOptions,
  compilation: &Compilation,
) -> RuntimeSpec {
  if let Some(depend_on) = &options.depend_on {
    let mut result: RuntimeSpec = Default::default();
    let mut queue = vec![];
    queue.extend(depend_on.clone());

    while let Some(name) = queue.pop() {
      let Some(entry_point_ukey) = compilation.entrypoints.get(&name) else {
        continue;
      };
      let entry_point = compilation.chunk_group_by_ukey.expect_get(entry_point_ukey);
      let Some(entry_options) = entry_point.kind.get_entry_options() else {
        continue;
      };

      if let Some(depend_on) = &entry_options.depend_on {
        for depend in depend_on {
          queue.push(depend.clone());
        }
      } else {
        result = merge_runtime(
          &result,
          &compilation.get_entry_runtime(
            &entry_options.runtime.clone().unwrap_or(name),
            Some(entry_options),
          ),
        );
      }
    }
    result
  } else {
    RuntimeSpec::from_iter([Arc::from(
      options.runtime.clone().unwrap_or_else(|| name.to_string()),
    )])
  }
}

static HASH_REPLACERS: Lazy<Vec<(&Lazy<Regex>, &str)>> = Lazy::new(|| {
  vec![
    (&HASH_PLACEHOLDER, "[hash]"),
    (&FULL_HASH_PLACEHOLDER, "[fullhash]"),
    (&CHUNK_HASH_PLACEHOLDER, "[chunkhash]"),
    (&CONTENT_HASH_PLACEHOLDER, "[contenthash]"),
  ]
});

pub fn get_filename_without_hash_length<F: Clone>(
  filename: &Filename<F>,
) -> (Filename<F>, HashMap<String, usize>) {
  let mut hash_len_map = HashMap::new();
  let Some(template) = filename.template() else {
    return (filename.clone(), hash_len_map);
  };
  let mut template = template.to_string();
  for (reg, key) in HASH_REPLACERS.iter() {
    template = reg
      .replace_all(&template, |caps: &Captures| {
        if let Some(hash_len) = match caps.get(2) {
          Some(m) => m.as_str().parse().ok(),
          None => None,
        } {
          hash_len_map.insert(key.to_string(), hash_len);
        }
        key
      })
      .into_owned();
  }
  (Filename::from(template), hash_len_map)
}
