use tracing::instrument;

use crate::{plugin_driver::PluginDriver, Chunk, OutputChunk, PluginRenderChunkHookOutput};

#[instrument(skip_all)]
pub fn render_chunk(
  output_chunk: OutputChunk,
  chunk: &Chunk,
  plugin_driver: &PluginDriver,
) -> PluginRenderChunkHookOutput {
  plugin_driver.render_chunk(output_chunk, chunk)
}
