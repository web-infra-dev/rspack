use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, Plugin, PluginContext, PluginRuntimeRequirementsInTreeOutput,
  RuntimeGlobals, RuntimeModule, RuntimeRequirementsInTreeArgs,
};
use rspack_identifier::Identifier;

use crate::ShareRuntimeModule;

#[derive(Debug, Default)]
pub struct ModuleFederationRuntimePlugin;

impl Plugin for ModuleFederationRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.ModuleFederationRuntimePlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    if args
      .runtime_requirements
      .contains(RuntimeGlobals::SHARE_SCOPE_MAP)
    {
      args
        .compilation
        .add_runtime_module(args.chunk, Box::<ModuleFederationRuntimeModule>::default());
      args
        .compilation
        .add_runtime_module(args.chunk, Box::<ShareRuntimeModule>::default());
    }
    Ok(())
  }
}

#[derive(Debug, Eq)]
pub struct ModuleFederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ModuleFederationRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/module_federation_runtime"),
      chunk: None,
    }
  }
}

impl RuntimeModule for ModuleFederationRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> BoxSource {
    RawSource::from(format!(r#"{}.MF = {{}};"#, RuntimeGlobals::REQUIRE)).boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(ModuleFederationRuntimeModule);
