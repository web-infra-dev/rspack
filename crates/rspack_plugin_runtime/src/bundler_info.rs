use rspack_core::{
  Plugin, PluginContext, PluginRuntimeRequirementsInTreeOutput, RuntimeGlobals,
  RuntimeRequirementsInTreeArgs,
};
use rustc_hash::FxHashSet;

use crate::runtime_module::RspackVersionRuntimeModule;

#[derive(Debug)]
pub enum BundlerInfoMode {
  Auto,
  All,
  Partial(FxHashSet<String>),
}

#[derive(Debug)]
pub struct BundlerInfoPlugin {
  version: String,
  mode: BundlerInfoMode,
}

impl BundlerInfoPlugin {
  pub fn new(mode: BundlerInfoMode, version: String) -> Self {
    Self { version, mode }
  }
}

impl Plugin for BundlerInfoPlugin {
  fn name(&self) -> &'static str {
    "BundlerInfoPlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    if match &self.mode {
      BundlerInfoMode::All => true,
      BundlerInfoMode::Partial(s) => s.get("version").is_some(),
      BundlerInfoMode::Auto => args
        .runtime_requirements
        .contains(RuntimeGlobals::RSPACK_VERSION),
    } {
      args.compilation.add_runtime_module(
        args.chunk,
        Box::new(RspackVersionRuntimeModule::new(self.version.clone())),
      );
    }
    Ok(())
  }
}
