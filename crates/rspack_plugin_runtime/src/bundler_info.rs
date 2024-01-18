use rspack_core::{
  Plugin, PluginContext, PluginRuntimeRequirementsInTreeOutput, RuntimeGlobals,
  RuntimeRequirementsInTreeArgs,
};
use rustc_hash::FxHashSet;

use crate::runtime_module::RspackVersionRuntimeModule;

#[derive(Debug)]
pub enum BundlerInfoForceMode {
  Auto,
  All,
  Partial(FxHashSet<String>),
}

#[derive(Debug)]
pub struct BundlerInfoPlugin {
  version: String,
  force: BundlerInfoForceMode,
}

impl BundlerInfoPlugin {
  pub fn new(force: BundlerInfoForceMode, version: String) -> Self {
    Self { version, force }
  }
}

#[async_trait::async_trait]
impl Plugin for BundlerInfoPlugin {
  fn name(&self) -> &'static str {
    "BundlerInfoPlugin"
  }

  async fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    if match &self.force {
      BundlerInfoForceMode::All => true,
      BundlerInfoForceMode::Partial(s) => s.get("version").is_some(),
      BundlerInfoForceMode::Auto => args
        .runtime_requirements
        .contains(RuntimeGlobals::RSPACK_VERSION),
    } {
      args
        .compilation
        .add_runtime_module(
          args.chunk,
          Box::new(RspackVersionRuntimeModule::new(self.version.clone())),
        )
        .await;
    }
    Ok(())
  }
}
