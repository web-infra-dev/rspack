use async_trait::async_trait;
use rspack_error::Result;

use crate::{
  Compilation, build_chunk_graph::build_chunk_graph, compilation::pass::PassExt,
  incremental::IncrementalPasses, logger::Logger, use_code_splitting_cache,
};

pub struct BuildChunkGraphPass;

#[async_trait]
impl PassExt for BuildChunkGraphPass {
  fn name(&self) -> &'static str {
    "build chunk graph"
  }

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::BUILD_CHUNK_GRAPH
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let logger = compilation.get_logger("rspack.Compilation");
    compilation.module_graph_cache_artifact.freeze();
    use_code_splitting_cache(compilation, |compilation| async {
      let start = logger.time("rebuild chunk graph");
      build_chunk_graph(compilation)?;
      logger.time_end(start);
      Ok(compilation)
    })
    .await?;
    Ok(())
  }
}
