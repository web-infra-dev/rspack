use rspack_util::fx_hash::FxIndexMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{EntryData, EntryOptions, RuntimeSpec};

pub fn get_entry_runtime(
  name: &str,
  options: &EntryOptions,
  entries: &FxIndexMap<String, EntryData>,
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
        result.extend(&RuntimeSpec::from_entry(&name, options.runtime.as_ref()));
      }
    }
    result
  } else {
    RuntimeSpec::from_entry(name, options.runtime.as_ref())
  }
}
