use rspack_error::Result;

use crate::{Compilation, Logger, SharedPluginDriver};

/// Runs the optimization phase hooks: optimize_modules, after_optimize_modules, optimize_chunks
pub async fn optimize_chunks_phase(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let mut diagnostics = vec![];
  while matches!(
    plugin_driver
      .compilation_hooks
      .optimize_modules
      .call(compilation, &mut diagnostics)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeModules"))?,
    Some(true)
  ) {}
  compilation.extend_diagnostics(diagnostics);

  plugin_driver
    .compilation_hooks
    .after_optimize_modules
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterOptimizeModules"))?;

  while matches!(
    plugin_driver
      .compilation_hooks
      .optimize_chunks
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunks"))?,
    Some(true)
  ) {}

  Ok(())
}

/// Runs the tree optimization phase: optimize_tree, optimize_chunk_modules
pub async fn optimize_tree_phase(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize");

  plugin_driver
    .compilation_hooks
    .optimize_tree
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeTree"))?;

  plugin_driver
    .compilation_hooks
    .optimize_chunk_modules
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"))?;

  logger.time_end(start);
  Ok(())
}
