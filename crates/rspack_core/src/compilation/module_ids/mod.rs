use std::mem;

use rspack_error::Result;

use crate::{Compilation, Logger, SharedPluginDriver, reset_artifact_if_passes_disabled};

pub async fn module_ids(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("module ids");

  // Check if MODULE_IDS pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.module_ids_artifact,
  );

  let mut diagnostics = vec![];
  let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);
  plugin_driver
    .compilation_hooks
    .module_ids
    .call(compilation, &mut module_ids_artifact, &mut diagnostics)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;
  compilation.module_ids_artifact = module_ids_artifact;
  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}
