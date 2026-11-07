use async_trait::async_trait;
use rspack_error::{Diagnostic, Result};

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
    check_lazy_barrel_imports(compilation);
    Ok(())
  }
}

fn check_lazy_barrel_imports(compilation: &mut Compilation) {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let diagnostics = compilation
    .build_module_graph_artifact
    .module_to_lazy_make
    .modules_deferring_imports()
    .filter(|module| chunk_graph.get_number_of_module_chunks(**module) > 0)
    .map(|module| {
      Diagnostic::from(rspack_error::error!(
        "Lazy barrel deferred the imports of {module}, but the module is included in a chunk, so its imports are missing at runtime. This is a bug in Rspack, please report it."
      ))
    })
    .collect::<Vec<_>>();
  compilation.extend_diagnostics(diagnostics);
}
