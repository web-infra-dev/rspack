use async_trait::async_trait;
use rspack_error::Result;

use crate::{Compilation, cache::Cache, logger::Logger};

/// A compilation pass that transforms the compilation state.
///
/// Inspired by rustc's MirPass design. Each pass implements `run_pass`
/// with its core logic. The `run` method wraps execution with logging
/// and cache hooks (before_pass / after_pass).
#[async_trait]
pub trait PassExt: Send + Sync {
  /// The name of this pass, used for logging and profiling.
  fn name(&self) -> &'static str;

  /// Core pass logic.
  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()>;

  /// Override this instead of run_pass if you need cache access mid-pass.
  /// Default delegates to run_pass (ignoring cache).
  async fn run_pass_with_cache(
    &self,
    compilation: &mut Compilation,
    _cache: &mut dyn Cache,
  ) -> Result<()> {
    self.run_pass(compilation).await
  }

  /// Called before run_pass. For cache restore, artifact cleanup.
  async fn before_pass(&self, _compilation: &mut Compilation, _cache: &mut dyn Cache) {}

  /// Called after run_pass succeeds. For cache save.
  async fn after_pass(&self, _compilation: &mut Compilation, _cache: &mut dyn Cache) {}

  /// Whether this pass is enabled for this compilation.
  fn is_enabled(&self, _compilation: &Compilation) -> bool {
    true
  }

  /// Unified entry point: check enabled → log → before_pass → run_pass → after_pass
  async fn run(&self, compilation: &mut Compilation, cache: &mut dyn Cache) -> Result<()> {
    if !self.is_enabled(compilation) {
      return Ok(());
    }
    let logger = compilation.get_logger("rspack.Compilation");
    let start = logger.time(self.name());

    self.before_pass(compilation, cache).await;
    let result = self.run_pass_with_cache(compilation, cache).await;
    if result.is_ok() {
      self.after_pass(compilation, cache).await;
    }

    logger.time_end(start);
    result
  }
}
