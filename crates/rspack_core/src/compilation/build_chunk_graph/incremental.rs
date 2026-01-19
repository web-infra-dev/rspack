use rspack_error::Result;
use tracing::instrument;

use super::code_splitter::CodeSplitter;
use crate::{ChunkGroupUkey, Compilation};

impl CodeSplitter {
  #[instrument(skip_all)]
  pub(crate) fn remove_orphan(&mut self, compilation: &mut Compilation) -> Result<()> {
    let mut removed = vec![];
    for chunk_group in compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .values()
    {
      let ukey = chunk_group.ukey;
      if !chunk_group.kind.is_entrypoint() && chunk_group.parents.is_empty() {
        removed.push(ukey);
      }
    }

    for removed_cg in &removed {
      self.remove_chunk_group(*removed_cg, compilation)?;
    }

    if !removed.is_empty() {
      self.remove_orphan(compilation)?;
    }

    Ok(())
  }

  fn remove_chunk_group(
    &mut self,
    chunk_group_ukey: ChunkGroupUkey,
    compilation: &mut Compilation,
  ) -> Result<()> {
    let Some(cgi_ukey) = self.chunk_group_info_map.remove(&chunk_group_ukey) else {
      return Ok(());
    };

    let chunk_group_info = self
      .chunk_group_infos
      .remove(&cgi_ukey)
      .expect("when we have cgi ukey, we have cgi");

    for block in chunk_group_info.outgoing_blocks.iter() {
      if let Some(infos) = self.block_owner.get_mut(block) {
        infos.remove(&cgi_ukey);
      }
    }

    let chunk_group = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .remove(&chunk_group_ukey)
      .expect("when we have cgi, we have chunk group");

    let chunk_group_name = chunk_group.name().map(|s| s.to_string());
    if let Some(name) = &chunk_group_name {
      compilation
        .build_chunk_graph_artifact
        .named_chunk_groups
        .remove(name);
      compilation
        .build_chunk_graph_artifact
        .entrypoints
        .swap_remove(name);
      self.named_chunk_groups.remove(name);
      self.named_async_entrypoints.remove(name);
    }

    // remove child parent relations
    for child in chunk_group_info.children.iter() {
      let Some(child_cgi) = self.chunk_group_infos.get_mut(child) else {
        continue;
      };

      child_cgi.available_sources.swap_remove(&cgi_ukey);

      if let Some(child_cg) = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .get_mut(&child_cgi.chunk_group)
      {
        child_cg.parents.remove(&chunk_group_ukey);
      }
    }

    for parent in chunk_group.parents.iter() {
      let Some(parent_cg) = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .get_mut(parent)
      else {
        // maybe already removed
        continue;
      };

      parent_cg.children.swap_remove_full(&chunk_group_ukey);

      if let Some(parent_cgi) = self.chunk_group_info_map.get(parent)
        && let Some(parent_cgi) = self.chunk_group_infos.get_mut(parent_cgi)
      {
        parent_cgi.children.swap_remove(&cgi_ukey);
        parent_cgi.available_children.swap_remove(&cgi_ukey);
      }
    }

    let chunk_graph = &mut compilation.build_chunk_graph_artifact.chunk_graph;

    // remove cgc and cgm
    for chunk_ukey in chunk_group.chunks.iter() {
      let Some(chunk) = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get_mut(chunk_ukey)
      else {
        continue;
      };

      if chunk.remove_group(&chunk_group_ukey) && chunk.groups().is_empty() {
        // remove orphan chunk
        if let Some(name) = chunk.name() {
          compilation
            .build_chunk_graph_artifact
            .named_chunks
            .remove(name);
        }
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .remove(chunk_ukey);

        // remove cgc and cgm
        if let Some(chunk_graph_chunk) = chunk_graph.remove_chunk(chunk_ukey) {
          for &module_identifier in chunk_graph_chunk.modules() {
            let Some(cgm) = chunk_graph.get_chunk_graph_module_mut(module_identifier) else {
              continue;
            };

            if cgm.chunks.remove(chunk_ukey) && cgm.chunks.is_empty() {
              chunk_graph.remove_module(module_identifier)
            }
          }
        };

        // remove mask
        self.mask_by_chunk.remove(chunk_ukey);

        // try to remove runtime chunk, runtime chunk is also included by chunk_group.chunks
        self.runtime_chunks.remove(chunk_ukey);
      }
    }

    // remove chunk group
    compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .remove(&chunk_group_ukey);

    Ok(())
  }
}
