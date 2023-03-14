use dashmap::DashMap;
use rspack_core::{Chunk, ChunkGraphChunk, ChunkUkey, Plugin};
use rspack_identifier::Identifier;

/// In practice, the algorithm friendly to development/hmr of splitting chunks is doing nothing.
/// But there are number of duplicated modules in very large projects, which affects the performance of the development/hmr.
/// Currently, the plugin does following things:
/// - Split modules shared by multiple chunks into a new chunk.
#[derive(Debug, Default)]
pub struct DevFriendlySplitChunksPlugin;

impl DevFriendlySplitChunksPlugin {
  pub fn new() -> Self {
    Self
  }
}

struct SharedModule {
  module: Identifier,
  ref_chunks: Vec<ChunkUkey>,
}

struct ChunkInfo<'a> {
  modules: &'a [SharedModule],
}

impl Plugin for DevFriendlySplitChunksPlugin {
  fn name(&self) -> &'static str {
    "DevFriendlySplitChunksPlugin"
  }

  fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    use rayon::prelude::*;
    let compilation = args.compilation;
    let mut shared_modules = compilation
      .module_graph
      .modules()
      .par_bridge()
      .map(|m| m.identifier())
      .map(|module| {
        let chunks = compilation.chunk_graph.get_modules_chunks(module);
        SharedModule {
          module,
          ref_chunks: chunks.iter().cloned().collect(),
        }
      })
      .filter(|m| m.ref_chunks.len() > 1)
      .collect::<Vec<_>>();

    shared_modules.sort_unstable_by(|a, b| {
      // One's ref_count is greater, one should be put in front.
      let ret = b.ref_chunks.len().cmp(&a.ref_chunks.len());
      if ret != std::cmp::Ordering::Equal {
        return ret;
      }

      // If the len of ref_chunks is equal, fallback to compare module id.
      a.module.cmp(&b.module)
    });

    // The number doesn't go through deep consideration.
    const MAX_MODULES_PER_CHUNK: usize = 3000;

    // Yeah. Leaky abstraction, but fast.
    let module_to_chunk_graph_module = compilation
      .chunk_graph
      .chunk_graph_module_by_module_identifier
      .iter_mut()
      .collect::<DashMap<_, _>>();

    // Yeah. Leaky abstraction, but fast.
    let mut chunk_and_cgc = shared_modules
      .par_chunks(MAX_MODULES_PER_CHUNK)
      .map(|modules| {
        let mut chunk = Chunk::new(None, None, rspack_core::ChunkKind::Normal);
        chunk
          .chunk_reasons
          .push("Split with ref count> 1".to_string());
        let mut chunk_graph_chunk = ChunkGraphChunk::new();
        chunk_graph_chunk
          .modules
          .extend(modules.iter().map(|m| m.module));

        modules.iter().for_each(|module| {
          let mut cgm = module_to_chunk_graph_module
            .get_mut(&module.module)
            .expect("mgm should exist");
          cgm.chunks.insert(chunk.ukey);
        });

        let chunk_info = ChunkInfo { modules };

        (chunk_info, chunk, chunk_graph_chunk)
      })
      .collect::<Vec<_>>();

    std::mem::drop(module_to_chunk_graph_module);

    // Split old chunks.
    chunk_and_cgc.iter_mut().for_each(|(info, chunk, _cgc)| {
      info.modules.iter().for_each(|module| {
        module.ref_chunks.iter().for_each(|old_chunk| {
          old_chunk
            .as_mut(&mut compilation.chunk_by_ukey)
            .split(chunk, &mut compilation.chunk_group_by_ukey);
        });
      });
    });

    // Add new chunks to compilation.
    chunk_and_cgc.into_iter().for_each(|(_, chunk, cgc)| {
      compilation
        .chunk_graph
        .add_chunk_wit_chunk_graph_chunk(chunk.ukey, cgc);
      compilation.chunk_by_ukey.add(chunk);
    });

    // Remove shared modules from old chunks, since they are moved to new chunks.
    shared_modules.iter().for_each(|m| {
      m.ref_chunks.iter().for_each(|old_chunk| {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(old_chunk, m.module);
      });
    });

    Ok(())
  }
}
