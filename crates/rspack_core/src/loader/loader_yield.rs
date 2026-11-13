use std::sync::Arc;

use async_trait::async_trait;
use rspack_error::Result;
use rspack_loader_runner::{LoaderContext, LoaderYieldInput, LoaderYieldResult};

use super::RunnerContext;

#[async_trait]
pub trait NormalModuleLoaderStartYielding: Send + Sync {
  async fn run(
    &self,
    context: Arc<LoaderContext<RunnerContext>>,
    input: &mut Option<LoaderYieldInput>,
  ) -> Result<Option<LoaderYieldResult>>;

  fn stage(&self) -> i32 {
    0
  }
}

#[derive(Default)]
pub struct NormalModuleLoaderStartYieldingHook {
  taps: Vec<Box<dyn NormalModuleLoaderStartYielding>>,
}

impl std::fmt::Debug for NormalModuleLoaderStartYieldingHook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NormalModuleLoaderStartYieldingHook")
      .field("taps", &self.taps.len())
      .finish()
  }
}

impl NormalModuleLoaderStartYieldingHook {
  pub fn tap(&mut self, tap: impl NormalModuleLoaderStartYielding + 'static) {
    let index = self
      .taps
      .partition_point(|existing| existing.stage() <= tap.stage());
    self.taps.insert(index, Box::new(tap));
  }

  pub async fn call(
    &self,
    context: Arc<LoaderContext<RunnerContext>>,
    input: LoaderYieldInput,
  ) -> Result<LoaderYieldResult> {
    let mut input = Some(input);
    for tap in &self.taps {
      if let Some(update) = tap.run(Arc::clone(&context), &mut input).await? {
        return Ok(update);
      }
    }
    let input = input.expect("unhandled loader input");
    Ok(LoaderYieldResult::Unhandled(input))
  }
}
