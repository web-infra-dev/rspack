use rspack_core::{
  ChunkUkey, Compilation, CompilationRuntimeRequirementInTree, Plugin, PluginContext,
  RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;

use crate::runtime_module::RspackVersionRuntimeModule;

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
  force: BundlerInfoForceMode,
}

impl BundlerInfoPlugin {
  pub fn new(force: BundlerInfoForceMode, version: String) -> Self {
    Self::new_inner(version, force)
  }
}

#[plugin_hook(CompilationRuntimeRequirementInTree for BundlerInfoPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if match &self.force {
    BundlerInfoForceMode::All => true,
    BundlerInfoForceMode::Partial(s) => s.get("version").is_some(),
    BundlerInfoForceMode::Auto => runtime_requirements.contains(RuntimeGlobals::RSPACK_VERSION),
  } {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE);
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(RspackVersionRuntimeModule::new(self.version.clone())),
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
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
