use rspack_core::{
  BoxPlugin, ChunkUkey, Compilation, CompilationRuntimeRequirementInTree, Plugin, PluginContext,
  PluginExt, RuntimeGlobals, RuntimeModuleExt, WasmLoadingType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::AsyncWasmLoadingRuntimeModule;

pub fn enable_wasm_loading_plugin(wasm_loading_type: WasmLoadingType) -> BoxPlugin {
  match wasm_loading_type {
    WasmLoadingType::Fetch => FetchCompileAsyncWasmPlugin::default().boxed(),
    WasmLoadingType::AsyncNode => ReadFileCompileAsyncWasmPlugin::new(false).boxed(),
    WasmLoadingType::AsyncNodeModule => ReadFileCompileAsyncWasmPlugin::new(true).boxed(),
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct FetchCompileAsyncWasmPlugin;

#[plugin_hook(CompilationRuntimeRequirementInTree for FetchCompileAsyncWasmPlugin)]
fn fetch_compile_async_wasm_plugin_runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
    runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
    compilation.add_runtime_module(
      chunk_ukey,
      AsyncWasmLoadingRuntimeModule::new(
        format!("fetch({} + $PATH)", RuntimeGlobals::PUBLIC_PATH),
        true,
        *chunk_ukey,
      )
      .boxed(),
    )?;
  }

  Ok(None)
}

impl Plugin for FetchCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "FetchCompileAsyncWasmPlugin"
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
      .tap(fetch_compile_async_wasm_plugin_runtime_requirements_in_tree::new(self));
    Ok(())
  }
}

#[plugin]
#[derive(Debug)]
pub struct ReadFileCompileAsyncWasmPlugin {
  import: bool,
}

impl ReadFileCompileAsyncWasmPlugin {
  fn new(import: bool) -> Self {
    Self::new_inner(import)
  }
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ReadFileCompileAsyncWasmPlugin)]
fn read_file_compile_async_wasm_plugin_runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if runtime_requirements.contains(RuntimeGlobals::INSTANTIATE_WASM) {
    runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
    compilation.add_runtime_module(
      chunk_ukey,
      AsyncWasmLoadingRuntimeModule::new(
        if self.import {
          include_str!("runtime/read_file_compile_async_wasm_with_import.js").to_string()
        } else {
          include_str!("runtime/read_file_compile_async_wasm.js").to_string()
        },
        false,
        *chunk_ukey,
      )
      .boxed(),
    )?;
  }

  Ok(None)
}

impl Plugin for ReadFileCompileAsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "ReadFileCompileAsyncWasmPlugin"
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
      .tap(read_file_compile_async_wasm_plugin_runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
