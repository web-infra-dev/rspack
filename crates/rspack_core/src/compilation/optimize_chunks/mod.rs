use super::*;
use crate::logger::Logger;

pub async fn optimize_chunks_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize chunks");

  while matches!(
    plugin_driver
      .compilation_hooks
      .optimize_chunks
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunks"))?,
    Some(true)
  ) {}

  logger.time_end(start);
  Ok(())
}
