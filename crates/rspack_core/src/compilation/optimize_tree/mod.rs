use super::*;
use crate::logger::Logger;

pub async fn optimize_tree_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize tree");

  let result = plugin_driver
    .compilation_hooks
    .optimize_tree
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeTree"));

  logger.time_end(start);
  result
}
