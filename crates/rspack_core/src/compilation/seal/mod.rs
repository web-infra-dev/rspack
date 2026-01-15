use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver};

pub async fn seal_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  #[cfg(feature = "debug_tool")]
  {
    use rspack_util::debug_tool::wait_for_signal;
    wait_for_signal("seal compilation");
  }
  // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
  plugin_driver
    .compilation_hooks
    .seal
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.seal"))?;

  Ok(())
}
