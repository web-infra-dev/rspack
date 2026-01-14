use rspack_error::Result;

use crate::{Compilation, Logger, SharedPluginDriver};

pub async fn optimize_code_generation(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize code generation");

  plugin_driver
    .compilation_hooks
    .optimize_code_generation
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeCodeGeneration"))?;

  logger.time_end(start);
  Ok(())
}
