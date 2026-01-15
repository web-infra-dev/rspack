use super::*;
use crate::logger::Logger;

pub async fn optimize_chunk_modules_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize chunk modules");

  let result = plugin_driver
    .compilation_hooks
    .optimize_chunk_modules
    .call(compilation)
    .await
    .map(|_| ())
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"));

  logger.time_end(start);
  result
}
