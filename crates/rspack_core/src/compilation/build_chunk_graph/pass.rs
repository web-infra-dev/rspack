use crate::compilation::build_chunk_graph::{
  artifact::use_code_splitting_cache,
  build_chunk_graph,
};
use crate::compilation::Compilation;
use crate::logger::Logger;
use rspack_error::Result;

impl Compilation {
  pub async fn build_chunk_graph_pass(&mut self) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    self.module_graph_cache_artifact.freeze();
    use_code_splitting_cache(self, |compilation| async {
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
}
