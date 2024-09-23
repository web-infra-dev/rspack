use rspack_core::{BoxPlugin, ChunkLoading, ChunkLoadingType, PluginExt};
use rspack_plugin_javascript::JsPlugin;
use rspack_plugin_runtime::{
  CommonJsChunkLoadingPlugin, RuntimePlugin, StartupChunkDependenciesPlugin,
};

pub fn buildtime_plugins() -> Vec<BoxPlugin> {
  vec![
    RuntimePlugin::default().boxed(),
    JsPlugin::default().boxed(),
    // StartupChunkDependenciesPlugin::new(ChunkLoading::Enable(ChunkLoadingType::Require), false)
    //   .boxed(),
    // CommonJsChunkLoadingPlugin::new(false).boxed(),
  ]
}
