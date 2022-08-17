use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, Define};
use std::collections::HashMap;

pub type RawDefine = HashMap<String, String>;

impl RawOption<Define> for RawDefine {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Define> {
    Ok(self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
