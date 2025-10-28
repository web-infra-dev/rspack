use rspack_core::{BoxPlugin, ChunkLoadingType, PluginExt};
use rspack_plugin_runtime::{ArrayPushCallbackChunkFormatPlugin, enable_chunk_loading_plugin};

pub fn web_worker_template_plugin(plugins: &mut Vec<BoxPlugin>) {
  plugins.push(ArrayPushCallbackChunkFormatPlugin::default().boxed());
  // ImportScripts always uses async_chunk_loading: true (hardcoded in enable_chunk_loading_plugin)
  enable_chunk_loading_plugin(ChunkLoadingType::ImportScripts, true, plugins);
}
