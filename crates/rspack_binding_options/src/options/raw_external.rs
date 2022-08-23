use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, External};
use std::collections::HashMap;

pub type RawExternal = HashMap<String, String>;

impl RawOption<Vec<External>> for RawExternal {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Vec<External>> {
    Ok(vec![rspack_core::External::Object(self)])
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
