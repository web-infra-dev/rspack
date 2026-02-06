use std::sync::atomic::Ordering;

use rspack_error::Result;
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

use crate::{
  Compilation, compilation::build_module_graph::finish_build_module_graph, logger::Logger,
};

pub(super) async fn finish_module_graph_pass(compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");
  let start = logger.time("finish compilation");
  compilation.finish_build_module_graph().await?;

  logger.time_end(start);

  Ok(())
}

impl Compilation {
  #[instrument("Compilation:finish",target=TRACING_BENCH_TARGET, skip_all)]
  pub async fn finish_build_module_graph(&mut self) -> Result<()> {
    self.in_finish_make.store(false, Ordering::Release);
    // clean up the entry deps
    let make_artifact = self.build_module_graph_artifact.take();
    self
      .build_module_graph_artifact
      .replace(finish_build_module_graph(self, make_artifact).await?);
    // sync assets to module graph from module_executor
    if let Some(module_executor) = &mut self.module_executor {
      let mut module_executor = std::mem::take(module_executor);
      module_executor.after_build_module_graph(self).await?;
      self.module_executor = Some(module_executor);
    }
    // make finished, make artifact should be readonly thereafter.
    Ok(())
  }
}
