use super::*;
use crate::logger::Logger;

pub async fn optimize_code_generation_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize code generation");

  let mut build_module_graph_artifact = compilation.build_module_graph_artifact.take();
  let mut diagnostics = vec![];
  plugin_driver
    .compilation_hooks
    .optimize_code_generation
    .call(
      compilation,
      &mut build_module_graph_artifact,
      &mut diagnostics,
    )
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeCodeGeneration"))?;

  compilation
    .build_module_graph_artifact
    .replace(build_module_graph_artifact);
  compilation.extend_diagnostics(diagnostics);

  logger.time_end(start);
  Ok(())
}
