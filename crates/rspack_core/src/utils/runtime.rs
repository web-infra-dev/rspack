use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use crate::{
  EntryOptions, Filename, RuntimeSpec, CHUNK_HASH_PLACEHOLDER, CONTENT_HASH_PLACEHOLDER,
  FULL_HASH_PLACEHOLDER, HASH_PLACEHOLDER,
};

pub fn get_entry_runtime(name: &str, options: &EntryOptions) -> RuntimeSpec {
  RuntimeSpec::from_iter([Arc::from(
    options.runtime.clone().unwrap_or_else(|| name.to_string()),
  )])
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
