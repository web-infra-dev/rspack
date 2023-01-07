use std::collections::HashMap;

use rspack_core::{CompilerOptionsBuilder, External};

use crate::RawOption;

pub type RawExternal = HashMap<String, String>;

impl RawOption<Vec<External>> for RawExternal {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Vec<External>> {
    Ok(vec![rspack_core::External::Object(self)])
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
