use async_trait::async_trait;
use rspack_error::Result;

use crate::{Compilation, compilation::pass::PassExt};

pub struct SealPass;

#[async_trait]
impl PassExt for SealPass {
  fn name(&self) -> &'static str {
    "seal"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    #[cfg(feature = "debug_tool")]
    {
      use rspack_util::debug_tool::wait_for_signal;
      wait_for_signal("seal compilation");
    }
    // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .seal
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.seal"))?;

    Ok(())
  }
}
