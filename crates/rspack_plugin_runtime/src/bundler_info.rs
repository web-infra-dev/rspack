use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationRuntimeRequirementInTree, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;

use crate::runtime_module::{RspackUniqueIdRuntimeModule, RspackVersionRuntimeModule};

#[derive(Debug)]
pub enum BundlerInfoForceMode {
  Auto,
  All,
  Partial(FxHashSet<String>),
}

#[plugin]
#[derive(Debug)]
pub struct BundlerInfoPlugin {
  version: String,
  bundler_name: String,
  force: BundlerInfoForceMode,
}

impl BundlerInfoPlugin {
  pub fn new(version: String, bundler_name: String, force: BundlerInfoForceMode) -> Self {
    Self::new_inner(version, bundler_name, force)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for BundlerInfoPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  _compilation: &mut Compilation,
  _chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if match &self.force {
    BundlerInfoForceMode::All => true,
    BundlerInfoForceMode::Partial(s) => s.get("version").is_some(),
    BundlerInfoForceMode::Auto => runtime_requirements.contains(RuntimeGlobals::RSPACK_VERSION),
  } {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::RSPACK_VERSION);
  }

  if match &self.force {
    BundlerInfoForceMode::All => true,
    BundlerInfoForceMode::Partial(s) => s.get("uniqueId").is_some(),
    BundlerInfoForceMode::Auto => runtime_requirements.contains(RuntimeGlobals::RSPACK_UNIQUE_ID),
  } {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::RSPACK_UNIQUE_ID);
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for BundlerInfoPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  _runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::RSPACK_VERSION) {
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(RspackVersionRuntimeModule::new(self.version.clone())),
    )?;
  }

  if runtime_requirements.contains(RuntimeGlobals::RSPACK_UNIQUE_ID) {
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(RspackUniqueIdRuntimeModule::new(
        self.bundler_name.clone(),
        self.version.clone(),
      )),
    )?;
  }
  Ok(None)
}

impl Plugin for BundlerInfoPlugin {
  fn name(&self) -> &'static str {
    "BundlerInfoPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
