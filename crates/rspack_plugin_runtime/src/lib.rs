#![feature(get_mut_unchecked)]
mod helpers;
pub use helpers::*;
mod lazy_compilation;
pub use lazy_compilation::LazyCompilationPlugin;
mod common_js_chunk_format;
pub use common_js_chunk_format::CommonJsChunkFormatPlugin;
mod runtime_plugin;
use rspack_core::{BoxPlugin, ChunkLoading, ChunkLoadingType, PluginExt};
pub use runtime_plugin::RuntimePlugin;
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
pub use runtime_module::{
  chunk_has_css, is_enabled_for_chunk, stringify_chunks, GetChunkFilenameRuntimeModule,
};
mod startup_chunk_dependencies;
pub use startup_chunk_dependencies::StartupChunkDependenciesPlugin;
mod chunk_prefetch_preload;
pub use chunk_prefetch_preload::ChunkPrefetchPreloadPlugin;
mod bundler_info;
pub use bundler_info::{BundlerInfoForceMode, BundlerInfoPlugin};

pub fn enable_chunk_loading_plugin(loading_type: ChunkLoadingType, plugins: &mut Vec<BoxPlugin>) {
  match loading_type {
    ChunkLoadingType::Jsonp => {
      plugins.push(JsonpChunkLoadingPlugin.boxed());
    }
    ChunkLoadingType::Require => {
      plugins.push(
        StartupChunkDependenciesPlugin::new(ChunkLoading::Enable(ChunkLoadingType::Require), false)
          .boxed(),
      );
      plugins.push(CommonJsChunkLoadingPlugin::new(false).boxed())
    }
    ChunkLoadingType::AsyncNode => {
      plugins.push(
        StartupChunkDependenciesPlugin::new(
          ChunkLoading::Enable(ChunkLoadingType::AsyncNode),
          true,
        )
        .boxed(),
      );
      plugins.push(CommonJsChunkLoadingPlugin::new(true).boxed())
    }
    ChunkLoadingType::ImportScripts => {
      plugins.push(
        StartupChunkDependenciesPlugin::new(
          ChunkLoading::Enable(ChunkLoadingType::ImportScripts),
          true,
        )
        .boxed(),
      );
      plugins.push(ImportScriptsChunkLoadingPlugin.boxed());
    }
    ChunkLoadingType::Import => plugins.push(ModuleChunkLoadingPlugin.boxed()),
  }
}
