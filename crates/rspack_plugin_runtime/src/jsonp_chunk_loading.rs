use crate::runtime_module::JsonpChunkLoadingRuntimeModule;
use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
};
use rspack_error::Result;

#[derive(Debug)]
pub struct JsonPChunkLoadingPlugin {}

#[async_trait]
impl Plugin for JsonPChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "JsonPChunkLoadingPlugin"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(runtime_globals::ENSURE_CHUNK_HANDLERS) {
      runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY.to_string());
      runtime_requirements.insert(runtime_globals::HAS_OWN_PROPERTY.to_string());
      runtime_requirements.insert(runtime_globals::PUBLIC_PATH.to_string());
      runtime_requirements.insert(runtime_globals::LOAD_SCRIPT.to_string());
      runtime_requirements.insert(runtime_globals::GET_CHUNK_SCRIPT_FILENAME.to_string());
      compilation.add_runtime_module(chunk, JsonpChunkLoadingRuntimeModule::default().boxed());
    }

    Ok(())
  }
}
