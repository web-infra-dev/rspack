use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver, logger::Logger};

pub(super) async fn make_hook_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");

  let start = logger.time("make hook");
  plugin_driver.compiler_hooks.make.call(compilation).await?;
  logger.time_end(start);

  Ok(())
}
