use std::fmt::Debug;

use rspack_error::Result;
pub use rspack_swc_visitors::Provide;

use crate::{ApplyContext, CompilerOptions, Plugin, PluginContext};

#[derive(Debug)]
pub struct ProvidePlugin {
  options: Provide,
}

impl ProvidePlugin {
  pub fn new(options: Provide) -> Self {
    Self { options }
  }
}

impl Plugin for ProvidePlugin {
  fn name(&self) -> &'static str {
    "rspack.ProvidePlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.provide.extend(self.options.clone());
    Ok(())
  }
}

#[derive(Debug, Clone, Default)]
pub struct DecoratorOptions {
  // https://swc.rs/docs/configuration/compilation#jsctransformlegacydecorator
  pub legacy: bool,
  // https://swc.rs/docs/configuration/compilation#jsctransformdecoratormetadata
  pub emit_metadata: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Builtins {
  // TODO: refactor to string-replacement based
  pub provide: Provide,
}

#[derive(Debug, Clone, Default)]
pub struct PresetEnv {
  pub targets: Vec<String>,
  pub mode: Option<swc_core::ecma::preset_env::Mode>,
  pub core_js: Option<String>,
}
