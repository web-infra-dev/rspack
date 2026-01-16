use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver, logger::Logger};

pub async fn finish_make_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");
  let start = logger.time("finish make hook");
  plugin_driver
    .compiler_hooks
    .finish_make
    .call(compilation)
    .await?;
  logger.time_end(start);

  Ok(())
}
