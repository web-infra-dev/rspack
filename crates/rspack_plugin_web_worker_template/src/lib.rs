use rspack_core::{BoxPlugin, ChunkLoadingType, PluginExt};
use rspack_plugin_runtime::{ArrayPushCallbackChunkFormatPlugin, enable_chunk_loading_plugin};

pub fn web_worker_template_plugin(plugins: &mut Vec<BoxPlugin>) {
  plugins.push(ArrayPushCallbackChunkFormatPlugin::default().boxed());
  enable_chunk_loading_plugin(&ChunkLoadingType::ImportScripts, plugins);
}
