use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver};

pub async fn make_hook_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  plugin_driver.compiler_hooks.make.call(compilation).await
}
