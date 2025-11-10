mod helpers;
pub use helpers::*;
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
  EXPORT_WEBPACK_REQUIRE_RUNTIME_MODULE_ID, GetChunkFilenameRuntimeModule, chunk_has_css,
  chunk_has_js, is_enabled_for_chunk, stringify_chunks,
};
mod startup_chunk_dependencies;
pub use startup_chunk_dependencies::StartupChunkDependenciesPlugin;
mod chunk_prefetch_preload;
pub use chunk_prefetch_preload::ChunkPrefetchPreloadPlugin;
mod bundler_info;
pub use bundler_info::{BundlerInfoForceMode, BundlerInfoPlugin};
mod runtime_module_from_js;
pub use runtime_module_from_js::RuntimeModuleFromJs;
mod drive;
pub use drive::*;

pub fn enable_chunk_loading_plugin(
  loading_type: ChunkLoadingType,
  mf_async_startup: bool,
  plugins: &mut Vec<BoxPlugin>,
) {
  match loading_type {
    ChunkLoadingType::Jsonp => {
      if mf_async_startup {
        plugins.push(
          StartupChunkDependenciesPlugin::new(ChunkLoading::Enable(ChunkLoadingType::Jsonp), true)
            .boxed(),
        );
      }
      plugins.push(JsonpChunkLoadingPlugin::default().boxed());
    }
    ChunkLoadingType::Require => {
      plugins.push(
        StartupChunkDependenciesPlugin::new(
          ChunkLoading::Enable(ChunkLoadingType::Require),
          mf_async_startup,
        )
        .boxed(),
      );
      plugins.push(CommonJsChunkLoadingPlugin::new(mf_async_startup).boxed())
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
      plugins.push(ImportScriptsChunkLoadingPlugin::default().boxed());
    }
    ChunkLoadingType::Import => plugins.push(ModuleChunkLoadingPlugin::default().boxed()),
    ChunkLoadingType::Custom(_) => (),
  }
}
