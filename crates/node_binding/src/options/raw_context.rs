use crate::RawOption;
use rspack_core::{CompilerOptionsBuilder, Context};

pub type RawContext = String;

impl RawOption<Context> for RawContext {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Context> {
    Ok(self)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    std::env::current_dir().unwrap().display().to_string()
  }
}
