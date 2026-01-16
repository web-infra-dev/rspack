use rspack_error::Result;

use crate::{
  Compilation, build_chunk_graph::build_chunk_graph, logger::Logger, use_code_splitting_cache,
};

pub async fn build_chunk_graph_pass(compilation: &mut Compilation) -> Result<()> {
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
