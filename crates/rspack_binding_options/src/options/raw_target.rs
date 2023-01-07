use rspack_core::{CompilerOptionsBuilder, Target};

use crate::RawOption;

pub type RawTarget = Vec<String>;

impl RawOption<Target> for RawTarget {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Target> {
    Target::new(&self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    vec![String::from("web")]
  }
}
