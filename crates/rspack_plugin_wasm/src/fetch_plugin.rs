use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Module, ModuleType, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
};

use crate::AsyncWasmRuntimeModule;

// TODO: for ChunkLoading
// #[derive(Debug)]
// pub struct FetchCompileWasmPlugin;

#[derive(Debug)]
pub struct FetchCompileAsyncWasmPlugin;

#[async_trait::async_trait]
impl Plugin for FetchCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "FetchCompileWasmPlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(runtime_globals::INSTANTIATE_WASM) {
      let compilation = &args.compilation;
      let chunk = args.chunk;
      let chunk_graph = &compilation.chunk_graph;
      let has_wasm_in_graph = chunk_graph
        .get_chunk_modules(chunk, &compilation.module_graph)
        .iter()
        .any(|module| *module.module_type() == ModuleType::WasmAsync);

      if !has_wasm_in_graph {
        return Ok(());
      }

      runtime_requirements.insert(runtime_globals::PUBLIC_PATH);
      args
        .compilation
        .add_runtime_module(args.chunk, AsyncWasmRuntimeModule::default().boxed());
    }

    Ok(())
  }
}
