use crate::runtime_module::{EnsureChunkRuntimeModule, OnChunkLoadedRuntimeModule};
use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, NormalRuntimeModule, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt, TargetPlatform,
};
use rspack_error::Result;

mod basic_runtime_requirements;
pub use basic_runtime_requirements::BasicRuntimeRequirementPlugin;
mod hot_module_replacement;
pub use hot_module_replacement::HotModuleReplacementPlugin;
mod css_modules;
pub use css_modules::CssModulesPlugin;
mod array_push_callback_chunk_format;
pub use array_push_callback_chunk_format::ArrayPushCallbackChunkFormatPlugin;
mod common_js_chunk_loading;
pub use common_js_chunk_loading::CommonJsChunkLoadingPlugin;
mod jsonp_chunk_loading;
pub use jsonp_chunk_loading::JsonPChunkLoadingPlugin;
mod runtime_module;

#[derive(Debug)]
pub struct RuntimePlugin {}

impl RuntimePlugin {}

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
    chunks.sort_by_key(|c| &c.id);
    for chunk in &chunks {
      if !chunk.is_only_initial(&compilation.chunk_group_by_ukey) {
        // TODO: use module async block instead of it at code generation
        runtime_requirements.insert(runtime_globals::ENSURE_CHUNK.to_string());
      }
    }

    compilation.add_runtime_module(
      chunk,
      NormalRuntimeModule::new(
        ("_rspack_require.js").to_string(),
        include_str!("runtime/require.js").to_string().replace(
          "GLOBAL",
          if matches!(
            compilation.options.target.platform,
            TargetPlatform::Web | TargetPlatform::None
          ) {
            "self"
          } else {
            "this"
          },
        ),
      )
      .boxed(),
    );

    // workaround for jsonp_chunk_loading can scan `ENSURE_CHUNK` to add additional runtime_requirements
    if runtime_requirements.contains(runtime_globals::ENSURE_CHUNK) {
      runtime_requirements.insert(runtime_globals::ENSURE_CHUNK_HANDLERS.to_string());
      compilation.add_runtime_module(chunk, EnsureChunkRuntimeModule::new(true).boxed());
    }

    if runtime_requirements.contains(runtime_globals::ON_CHUNKS_LOADED) {
      compilation.add_runtime_module(chunk, OnChunkLoadedRuntimeModule::default().boxed());
    }

    Ok(())
  }
}
