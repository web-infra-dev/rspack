use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver};

pub async fn finish_make_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  plugin_driver
    .compiler_hooks
    .finish_make
    .call(compilation)
    .await
}
