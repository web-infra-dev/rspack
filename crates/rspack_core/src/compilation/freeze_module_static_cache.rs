use async_trait::async_trait;
use rspack_error::Result;

use crate::{Compilation, compilation::pass::PassExt};

pub struct FreezeModuleStaticCachePass;

#[async_trait]
impl PassExt for FreezeModuleStaticCachePass {
  fn name(&self) -> &'static str {
    "freeze module static cache"
  }

  fn is_enabled(&self, compilation: &Compilation) -> bool {
    !compilation.options.mode.is_development()
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    compilation.module_static_cache_artifact.freeze();
    Ok(())
  }
}

pub struct UnfreezeModuleStaticCachePass;

#[async_trait]
impl PassExt for UnfreezeModuleStaticCachePass {
  fn name(&self) -> &'static str {
    "unfreeze module static cache"
  }

  fn is_enabled(&self, compilation: &Compilation) -> bool {
    !compilation.options.mode.is_development()
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    compilation.module_static_cache_artifact.unfreeze();
    Ok(())
  }
}
