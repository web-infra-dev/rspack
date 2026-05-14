use std::future::Future;

use rspack_error::Result;

use crate::{Compilation, cache::Cache, logger::Logger};

/// A compilation pass that transforms the compilation state.
///
/// Inspired by rustc's MirPass design. Each pass implements `run_pass`
/// with its core logic. The `run` method wraps execution with logging
/// and cache hooks (before_pass / after_pass).
pub trait PassExt: Send + Sync {
  /// The name of this pass, used for logging and profiling.
  fn name(&self) -> &'static str;

  /// Core pass logic.
  fn run_pass<'a>(
    &'a self,
    compilation: &'a mut Compilation,
  ) -> impl Future<Output = Result<()>> + Send + 'a;

  /// Override this instead of run_pass if you need cache access mid-pass.
  /// Default delegates to run_pass (ignoring cache).
  fn run_pass_with_cache<'a>(
    &'a self,
    compilation: &'a mut Compilation,
    _cache: &'a mut dyn Cache,
  ) -> impl Future<Output = Result<()>> + Send + 'a {
    async move { self.run_pass(compilation).await }
  }

  /// Called before run_pass. For cache restore, artifact cleanup.
  fn before_pass<'a>(
    &'a self,
    _compilation: &'a mut Compilation,
    _cache: &'a mut dyn Cache,
  ) -> impl Future<Output = ()> + Send + 'a {
    async {}
  }

  /// Called after run_pass succeeds. For cache save.
  fn after_pass<'a>(
    &'a self,
    _compilation: &'a mut Compilation,
    _cache: &'a mut dyn Cache,
  ) -> impl Future<Output = ()> + Send + 'a {
    async {}
  }

  /// Whether this pass is enabled for this compilation.
  fn is_enabled(&self, _compilation: &Compilation) -> bool {
    true
  }

  /// Unified entry point: check enabled → log → before_pass → run_pass → after_pass
  fn run<'a>(
    &'a self,
    compilation: &'a mut Compilation,
    cache: &'a mut dyn Cache,
  ) -> impl Future<Output = Result<()>> + Send + 'a {
    async move {
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
}
