#![feature(get_mut_unchecked)]

use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;
use runtime_module::AsyncRuntimeModule;

use crate::runtime_module::EnsureChunkRuntimeModule;
mod helpers;
pub use helpers::*;
mod lazy_compilation;
pub use lazy_compilation::LazyCompilationPlugin;
mod basic_runtime_requirements;
mod common_js_chunk_format;
pub use basic_runtime_requirements::BasicRuntimeRequirementPlugin;
pub use common_js_chunk_format::CommonJsChunkFormatPlugin;
mod hot_module_replacement;
pub use hot_module_replacement::HotModuleReplacementPlugin;
mod css_modules;
pub use css_modules::CssModulesPlugin;
mod array_push_callback_chunk_format;
pub use array_push_callback_chunk_format::ArrayPushCallbackChunkFormatPlugin;
mod common_js_chunk_loading;
pub use common_js_chunk_loading::CommonJsChunkLoadingPlugin;
mod jsonp_chunk_loading;
pub use jsonp_chunk_loading::JsonpChunkLoadingPlugin;
mod module_chunk_format;
pub use module_chunk_format::ModuleChunkFormatPlugin;
mod module_chunk_loading;
pub use module_chunk_loading::ModuleChunkLoadingPlugin;
mod import_scripts_chunk_loading;
pub use import_scripts_chunk_loading::ImportScriptsChunkLoadingPlugin;
mod runtime_module;
mod startup_chunk_dependencies;
pub use startup_chunk_dependencies::StartupChunkDependenciesPlugin;

#[derive(Debug)]
pub struct RuntimePlugin;

#[async_trait]
impl Plugin for RuntimePlugin {
  fn name(&self) -> &'static str {
    "RuntimePlugin"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;

    let mut chunks = compilation.chunk_by_ukey.values().collect::<Vec<_>>();
    chunks.sort_unstable_by_key(|c| &c.id);
    for chunk in &chunks {
      if !chunk.is_only_initial(&compilation.chunk_group_by_ukey) {
        // TODO: use module async block instead of it at code generation
        runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
      }
    }

    // workaround for jsonp_chunk_loading can scan `ENSURE_CHUNK` to add additional runtime_requirements
    if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK) {
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
      compilation.add_runtime_module(chunk, EnsureChunkRuntimeModule::new(true).boxed());
    }

    if runtime_requirements.contains(RuntimeGlobals::ASYNC_MODULE) {
      compilation.add_runtime_module(chunk, AsyncRuntimeModule::default().boxed());
    }

    Ok(())
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    for identifier in args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_iterable(args.chunk_ukey)
    {
      if let Some((hash, _)) = args
        .compilation
        .runtime_module_code_generation_results
        .get(identifier)
      {
        hash.hash(&mut args.hasher);
      }
    }
    Ok(())
  }
}
