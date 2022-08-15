use napi_derive::napi;
use serde::Deserialize;

use rspack_core::{CompilerOptionsBuilder, ModuleOptions};

use crate::RawOption;

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawModuleRule {}

#[derive(Deserialize, Default, Debug)]
#[napi(object)]
pub struct RawModuleOptions {
  pub rules: Vec<RawModuleRule>,
}

impl RawOption<ModuleOptions> for RawModuleOptions {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<ModuleOptions> {
    // FIXME: temporary implementation
    Ok(ModuleOptions::default())
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawModuleOptions { rules: vec![] }
  }
}
