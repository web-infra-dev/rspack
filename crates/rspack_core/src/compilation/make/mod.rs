use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver, cache::Cache, logger::Logger};

pub async fn make_hook_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
  cache: &mut dyn Cache,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");

  cache
    .before_build_module_graph(&mut compilation.build_module_graph_artifact)
    .await;

  let start = logger.time("make hook");
  plugin_driver.compiler_hooks.make.call(compilation).await?;
  logger.time_end(start);

  Ok(())
}
