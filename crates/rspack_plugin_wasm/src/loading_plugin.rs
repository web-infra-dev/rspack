use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, BoxPlugin, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginExt, RuntimeGlobals,
  RuntimeModuleExt, WasmLoadingType,
};

use crate::AsyncWasmLoadingRuntimeModule;

pub fn enable_wasm_loading_plugin(wasm_loading_type: WasmLoadingType) -> BoxPlugin {
  match wasm_loading_type {
    WasmLoadingType::Fetch => FetchCompileAsyncWasmPlugin.boxed(),
    WasmLoadingType::AsyncNode => ReadFileCompileAsyncWasmPlugin::new(false).boxed(),
    WasmLoadingType::AsyncNodeModule => ReadFileCompileAsyncWasmPlugin::new(true).boxed(),
  }
}

#[derive(Debug)]
pub struct FetchCompileAsyncWasmPlugin;

#[async_trait::async_trait]
impl Plugin for FetchCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "FetchCompileAsyncWasmPlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
      runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
      args.compilation.add_runtime_module(
        args.chunk,
        AsyncWasmLoadingRuntimeModule::new(format!(
          "fetch({} + $PATH)",
          RuntimeGlobals::PUBLIC_PATH
        ))
        .boxed(),
      );
    }

    Ok(())
  }
}

#[derive(Debug)]
pub struct ReadFileCompileAsyncWasmPlugin {
  import: bool,
}

impl ReadFileCompileAsyncWasmPlugin {
  fn new(import: bool) -> Self {
    Self { import }
  }
}

#[async_trait::async_trait]
impl Plugin for ReadFileCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "ReadFileCompileAsyncWasmPlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
      runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
      args.compilation.add_runtime_module(
        args.chunk,
        AsyncWasmLoadingRuntimeModule::new(if self.import {
          include_str!("runtime/read_file_compile_async_wasm_with_import.js").to_string()
        } else {
          include_str!("runtime/read_file_compile_async_wasm.js").to_string()
        })
        .boxed(),
      );
    }

    Ok(())
  }
}
