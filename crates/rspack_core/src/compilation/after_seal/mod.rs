use super::*;
use crate::logger::Logger;

pub async fn after_seal_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("after seal");
  compilation.after_seal(plugin_driver).await?;
  logger.time_end(start);
  Ok(())
}

impl Compilation {
  #[instrument("Compilation:after_seal", target=TRACING_BENCH_TARGET,skip_all)]
  async fn after_seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver.compilation_hooks.after_seal.call(self).await
  }
}
