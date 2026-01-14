use rspack_error::{Diagnostic, Result};
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

use crate::{Compilation, Logger, SharedPluginDriver};

#[instrument("Compilation:process_assets", target = TRACING_BENCH_TARGET, skip_all)]
pub async fn process_assets(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("process assets");

  let result = plugin_driver
    .compilation_hooks
    .process_assets
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"));

  logger.time_end(start);
  result
}

#[instrument("Compilation:after_process_assets", skip_all)]
pub async fn after_process_assets(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("after process assets");

  let mut diagnostics: Vec<Diagnostic> = vec![];

  let res = plugin_driver
    .compilation_hooks
    .after_process_assets
    .call(compilation, &mut diagnostics)
    .await;

  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  res
}

#[instrument("Compilation:after_seal", target = TRACING_BENCH_TARGET, skip_all)]
pub async fn after_seal(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("after seal");

  let result = plugin_driver
    .compilation_hooks
    .after_seal
    .call(compilation)
    .await;

  logger.time_end(start);
  result
}
