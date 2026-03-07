use std::sync::atomic::Ordering;

use rspack_error::Result;
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

use crate::{
  Compilation, compilation::build_module_graph::finish_build_module_graph, logger::Logger,
};

pub async fn finish_module_graph_pass(compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");
  let start = logger.time("finish compilation");
  finish_build_module_graph_pass(compilation).await?;

  logger.time_end(start);

  Ok(())
}

#[instrument("Compilation:finish",target=TRACING_BENCH_TARGET, skip_all)]
pub async fn finish_build_module_graph_pass(compilation: &mut Compilation) -> Result<()> {
  compilation.in_finish_make.store(false, Ordering::Release);
  // clean up the entry deps
  let mut make_artifact = compilation.build_module_graph_artifact.steal();
  compilation.set_module_executor(make_artifact.module_executor.take());
  let exports_info_artifact = compilation.exports_info_artifact.steal();
  let (mut make_artifact, exports_info_artifact) =
    finish_build_module_graph(compilation, make_artifact, exports_info_artifact).await?;
  make_artifact.module_executor = compilation.take_module_executor();
  compilation.build_module_graph_artifact = make_artifact.into();
  compilation.exports_info_artifact = exports_info_artifact.into();
  // sync assets to module graph from module_executor
  if let Some(mut module_executor) = compilation.take_module_executor() {
    module_executor
      .after_build_module_graph(compilation)
      .await?;
    compilation.set_module_executor(Some(module_executor));
  }
  // make finished, make artifact should be readonly thereafter.
  Ok(())
}
