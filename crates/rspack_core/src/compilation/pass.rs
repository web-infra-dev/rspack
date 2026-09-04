use async_trait::async_trait;
use rspack_error::Result;

use crate::{
  Compilation, artifacts::IncrementalArtifacts, incremental::IncrementalPasses,
  legacy_cache::Cache, logger::Logger,
};

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

  /// Artifact groups associated with this pass in the compiler pipeline.
  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::empty()
  }

  /// Called before run_pass for build cache restore.
  async fn before_pass(&self, _compilation: &mut Compilation, _cache: &mut dyn Cache) {}

  /// Called after run_pass succeeds. For cache save.
  async fn after_pass(&self, _compilation: &mut Compilation, _cache: &mut dyn Cache) -> Result<()> {
    Ok(())
  }

  /// Whether this pass is enabled for this compilation.
  fn is_enabled(&self, _compilation: &Compilation) -> bool {
    true
  }

  /// Unified entry point: check enabled → log → before_pass → run_pass → after_pass
  async fn run(&self, compilation: &mut Compilation, cache: &mut dyn Cache) -> Result<()> {
    run(self, compilation, None, cache).await
  }
}

pub(crate) async fn run_with_incremental_artifacts(
  pass: &dyn PassExt,
  compilation: &mut Compilation,
  incremental_artifacts: &mut IncrementalArtifacts,
  cache: &mut dyn Cache,
) -> Result<()> {
  run(pass, compilation, Some(incremental_artifacts), cache).await
}

async fn run<P: PassExt + ?Sized>(
  pass: &P,
  compilation: &mut Compilation,
  mut incremental_artifacts: Option<&mut IncrementalArtifacts>,
  cache: &mut dyn Cache,
) -> Result<()> {
  if !pass.is_enabled(compilation) {
    return Ok(());
  }
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time(pass.name());

  let incremental_passes = pass.incremental_passes();
  if let Some(incremental_artifacts) = incremental_artifacts.as_deref_mut() {
    incremental_artifacts.recover(incremental_passes, compilation);
  }
  pass.before_pass(compilation, cache).await;
  let result = match pass.run_pass_with_cache(compilation, cache).await {
    Ok(()) => match pass.after_pass(compilation, cache).await {
      Ok(()) => {
        if let Some(incremental_artifacts) = incremental_artifacts {
          incremental_artifacts.capture(incremental_passes, compilation);
        }
        Ok(())
      }
      Err(error) => Err(error),
    },
    Err(error) => Err(error),
  };

  logger.time_end(start);
  result
}
