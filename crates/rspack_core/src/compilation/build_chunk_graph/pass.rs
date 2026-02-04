use async_trait::async_trait;
use rspack_error::Result;

use crate::{
  Compilation, build_chunk_graph::build_chunk_graph, cache::Cache, compilation::pass::PassExt,
  logger::Logger, use_code_splitting_cache,
};

pub struct BuildChunkGraphPass;

#[async_trait]
impl PassExt for BuildChunkGraphPass {
  fn name(&self) -> &'static str {
    "build chunk graph"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_build_chunk_graph(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let logger = compilation.get_logger("rspack.Compilation");
    compilation.module_graph_cache_artifact.freeze();
    use_code_splitting_cache(compilation, |compilation| async {
      let start = logger.time("rebuild chunk graph");
      build_chunk_graph(compilation)?;
      compilation
        .chunk_graph
        .generate_dot(compilation, "after-code-splitting")
        .await;
      logger.time_end(start);
      Ok(compilation)
    })
    .await?;
    Ok(())
  }

  async fn after_pass(&self, compilation: &Compilation, cache: &mut dyn Cache) {
    cache.after_build_chunk_graph(compilation).await;
  }
}
