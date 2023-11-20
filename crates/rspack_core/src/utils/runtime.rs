use std::sync::Arc;

use crate::{EntryOptions, RuntimeSpec};

pub fn get_entry_runtime(name: &str, options: &EntryOptions) -> RuntimeSpec {
  RuntimeSpec::from_iter([Arc::from(
    options.runtime.clone().unwrap_or_else(|| name.to_string()),
  )])
}
