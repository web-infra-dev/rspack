use async_trait::async_trait;
use rspack_error::Result;

use super::*;
use crate::compilation::pass::PassExt;

pub(super) struct AssignRuntimeIdsPass;

#[async_trait]
impl PassExt for AssignRuntimeIdsPass {
  fn name(&self) -> &'static str {
    "assign runtime ids"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    fn process_entrypoint(
      entrypoint_ukey: &ChunkGroupUkey,
      chunk_group_by_ukey: &ChunkGroupByUkey,
      chunk_by_ukey: &ChunkByUkey,
      chunk_graph: &mut ChunkGraph,
    ) {
      let entrypoint = chunk_group_by_ukey.expect_get(entrypoint_ukey);
      let runtime = entrypoint
        .kind
        .get_entry_options()
        .and_then(|o| match &o.runtime {
          Some(EntryRuntime::String(s)) => Some(s.to_owned()),
          _ => None,
        })
        .or(entrypoint.name().map(|n| n.to_string()));
      if let (Some(runtime), Some(chunk)) = (
        runtime,
        chunk_by_ukey.get(&entrypoint.get_runtime_chunk(chunk_group_by_ukey)),
      ) {
        chunk_graph.set_runtime_id(runtime, chunk.id().map(|id| id.to_string()));
      }
    }
    for i in compilation.build_chunk_graph_artifact.entrypoints.iter() {
      process_entrypoint(
        i.1,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
        &mut compilation.build_chunk_graph_artifact.chunk_graph,
      )
    }
    for i in compilation
      .build_chunk_graph_artifact
      .async_entrypoints
      .iter()
    {
      process_entrypoint(
        i,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
        &mut compilation.build_chunk_graph_artifact.chunk_graph,
      )
    }
    Ok(())
  }
}
