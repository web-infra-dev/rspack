use num_bigint::BigUint;
use rspack_collections::{IdentifierIndexSet, UkeySet};
use rspack_error::Result;
use tracing::instrument;

use super::code_splitter::{CgiUkey, CodeSplitter, DependenciesBlockIdentifier};
use crate::{
  is_runtime_equal, AsyncDependenciesBlockIdentifier, ChunkGroupUkey, ChunkUkey, Compilation,
  GroupOptions, ModuleIdentifier, RuntimeSpec,
};

#[derive(Debug, Clone)]
pub enum ChunkReCreation {
  Entry(String),
  Normal(NormalChunkRecreation),
}

#[derive(Debug, Clone)]
pub struct NormalChunkRecreation {
  pub block: AsyncDependenciesBlockIdentifier,
  module: ModuleIdentifier,
  pub cgi: CgiUkey,
  chunk: ChunkUkey,
}

impl ChunkReCreation {
  pub fn rebuild(self, splitter: &mut CodeSplitter, compilation: &mut Compilation) -> Result<()> {
    match self {
      ChunkReCreation::Entry(entry) => {
        let input = splitter.prepare_entry_input(&entry, compilation)?;
        splitter.prepare_entries(std::iter::once(input).collect(), compilation)?;
        Ok(())
      }
      ChunkReCreation::Normal(create_data) => {
        splitter.make_chunk_group(
          create_data.block,
          create_data.module,
          create_data.cgi,
          create_data.chunk,
          compilation,
        );
        Ok(())
      }
    }
  }
}

impl CodeSplitter {
  pub(crate) fn invalidate_from_module(
    &mut self,
    module: ModuleIdentifier,
    compilation: &mut Compilation,
  ) -> Result<Vec<ChunkReCreation>> {
    let chunk_graph = &mut compilation.chunk_graph;

    // Step 1. find all invalidate chunk groups and remove module from ChunkGraph
    let Some(cgm) = chunk_graph.get_chunk_graph_module_mut(module) else {
      return Ok(vec![]);
    };

    let invalidate_chunk_groups = cgm
      .chunks
      .iter()
      .flat_map(|chunk| {
        let chunk = compilation.chunk_by_ukey.expect_get(chunk);
        chunk.groups.clone()
      })
      .collect::<UkeySet<ChunkGroupUkey>>();

    chunk_graph.remove_module(module);

    let mut removed = vec![];
    for chunk_group_ukey in &invalidate_chunk_groups {
      if let Some(edges) = self.invalidate_chunk_group(*chunk_group_ukey, compilation)? {
        removed.extend(edges);
      };
    }

    Ok(removed)
  }

  pub(crate) fn invalidate_chunk_group(
    &mut self,
    chunk_group_ukey: ChunkGroupUkey,
    compilation: &mut Compilation,
  ) -> Result<Option<Vec<ChunkReCreation>>> {
    // prepare data
    let Some(cgi_ukey) = self.chunk_group_info_map.remove(&chunk_group_ukey) else {
      return Ok(None);
    };
    let Some(chunk_group_info) = self.chunk_group_infos.remove(&cgi_ukey) else {
      return Ok(None);
    };
    let Some(chunk_group) = compilation.chunk_group_by_ukey.remove(&chunk_group_ukey) else {
      return Ok(None);
    };

    let chunk_group_name = chunk_group.name().map(|s| s.to_string());
    if let Some(name) = &chunk_group_name {
      compilation.named_chunk_groups.remove(name);
      compilation.entrypoints.swap_remove(name);
      self.named_chunk_groups.remove(name);
      self.named_async_entrypoints.remove(name);
    }

    // remove child parent relations
    for child in chunk_group_info.children.iter() {
      let Some(child_cgi) = self.chunk_group_infos.get_mut(child) else {
        continue;
      };

      child_cgi.available_sources.swap_remove(&cgi_ukey);
      child_cgi.parents.swap_remove(&cgi_ukey);

      if let Some(child_cg) = compilation
        .chunk_group_by_ukey
        .get_mut(&child_cgi.chunk_group)
      {
        child_cg.parents.remove(&chunk_group_ukey);
      }
    }

    for parent in chunk_group.parents.iter() {
      let Some(parent_cg) = compilation.chunk_group_by_ukey.get_mut(parent) else {
        continue;
      };

      parent_cg.children.remove(&chunk_group_ukey);

      if let Some(parent_cgi) = self.chunk_group_info_map.get(parent) {
        if let Some(parent_cgi) = self.chunk_group_infos.get_mut(parent_cgi) {
          parent_cgi.children.swap_remove(&cgi_ukey);
          parent_cgi.available_children.swap_remove(&cgi_ukey);
        }
      }
    }

    let chunk_graph = &mut compilation.chunk_graph;

    // remove cgc and cgm
    for chunk_ukey in chunk_group.chunks.iter() {
      self.mask_by_chunk.remove(chunk_ukey);

      if let Some(chunk_graph_chunk) = chunk_graph.remove_chunk_graph_chunk(chunk_ukey) {
        for module_identifier in chunk_graph_chunk.modules {
          let Some(cgm) = chunk_graph.get_chunk_graph_module_mut(module_identifier) else {
            continue;
          };

          if cgm.chunks.remove(chunk_ukey) && cgm.chunks.is_empty() {
            chunk_graph.remove_module(module_identifier)
          }
        }
      };

      let Some(chunk) = compilation.chunk_by_ukey.get_mut(chunk_ukey) else {
        continue;
      };

      if chunk.groups.remove(&chunk_group_ukey) && chunk.groups.is_empty() {
        // remove orphan chunk
        if let Some(name) = &chunk.name {
          compilation.named_chunks.remove(name);
        }
        compilation.chunk_by_ukey.remove(chunk_ukey);
      }
    }

    // remove chunk group
    compilation.chunk_group_by_ukey.remove(&chunk_group_ukey);

    // remove runtime chunk
    if let Some(runtime_chunk) = chunk_group.runtime_chunk {
      self.runtime_chunks.remove(&runtime_chunk);
    }

    // remove data related to cgi
    // if let Some(blocks) = self.blocks_by_cgi.get(&cgi_ukey) {
    //   for block in blocks {
    //     self.block_chunk_groups.remove(block);
    //   }
    // }

    let mut edges = vec![];
    for (parent, blocks) in &chunk_group_info.parents {
      for block in blocks {
        self
          .block_chunk_groups
          .remove(&DependenciesBlockIdentifier::AsyncDependenciesBlock(*block));

        let Some(cache) = self.chunk_caches.get(&(
          *parent,
          DependenciesBlockIdentifier::AsyncDependenciesBlock(*block),
        )) else {
          continue;
        };

        let Some(parent_cg) = self
          .chunk_group_infos
          .get(parent)
          .and_then(|cgi| compilation.chunk_group_by_ukey.get(&cgi.chunk_group))
        else {
          continue;
        };

        edges.push(ChunkReCreation::Normal(NormalChunkRecreation {
          block: *block,
          module: cache.module,
          cgi: *parent,
          chunk: parent_cg.chunks[0],
        }));
      }
    }

    if let Some(blocks) = self.blocks_by_cgi.get(&cgi_ukey) {
      for block in blocks {
        self.block_chunk_groups.remove(block);
      }
    }

    self.stat_invalidated_chunk_group += 1;

    for child in chunk_group_info.children.iter() {
      let Some(child_cgi) = self.chunk_group_infos.get_mut(child) else {
        continue;
      };

      let cg = child_cgi.chunk_group;

      if let Some(child_edges) = self.invalidate_chunk_group(cg, compilation)? {
        edges.extend(child_edges);
      }
    }

    if let Some(name) = chunk_group_name
      && chunk_group.is_initial()
      && chunk_group.parents.is_empty()
    {
      return Ok(Some(vec![ChunkReCreation::Entry(name)]));
    }

    if edges.is_empty() {
      return Ok(None);
    }

    Ok(Some(edges))
  }

  #[instrument(skip_all)]
  pub(crate) fn remove_orphan(&mut self, compilation: &mut Compilation) -> Result<()> {
    let mut removed = vec![];
    for chunk_group in compilation.chunk_group_by_ukey.values() {
      let ukey = chunk_group.ukey;
      if !chunk_group.kind.is_entrypoint() && chunk_group.parents.is_empty() {
        removed.push(ukey);
      }
    }

    for removed_cg in &removed {
      self.invalidate_chunk_group(*removed_cg, compilation)?;
    }

    if !removed.is_empty() {
      self.remove_orphan(compilation)?;
    }

    Ok(())
  }

  pub fn recover_from_cache(&mut self, cgi_ukey: CgiUkey, compilation: &mut Compilation) -> bool {
    let Some(cgi) = self.chunk_group_infos.get(&cgi_ukey) else {
      return false;
    };
    if cgi.parents.len() != 1 {
      return false;
    }
    let Some(blocks) = self.blocks_by_cgi.get(&cgi.ukey) else {
      return false;
    };

    let cg = compilation.chunk_group_by_ukey.expect_get(&cgi.chunk_group);
    let chunk = cg.chunks[0];

    let Some(cache) = self
      .chunk_caches
      .get(&(
        *cgi.parents.iter().next().expect("should have one parent").0,
        *blocks.iter().next().expect("should have one block"),
      ))
      .cloned()
    else {
      return false;
    };

    let module_graph = compilation.get_module_graph();
    let DependenciesBlockIdentifier::AsyncDependenciesBlock(block_id) =
      blocks.iter().next().expect("should have one block")
    else {
      return false;
    };

    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");

    if !self.hit_cache(
      &cache,
      &cgi.runtime,
      cgi.min_available_modules.clone(),
      block.get_group_options(),
    ) {
      return false;
    }

    // update cache available modules
    self.outdated_chunk_group_info.insert(cgi_ukey);

    let cgi = self.chunk_group_infos.expect_get_mut(&cgi_ukey);
    cgi.skipped_items = cache.skipped_modules.clone();

    let chunk_graph = &mut compilation.chunk_graph;
    for module in cache.modules {
      let ordinal = self.get_module_ordinal(module);
      chunk_graph.connect_chunk_and_module(chunk, module);
      let mask = self.mask_by_chunk.entry(chunk).or_default();
      mask.set_bit(ordinal, true);

      // TODO: correct preorder index
    }

    for block in cache.outgoings {
      self.make_chunk_group(block, cache.module, cgi_ukey, chunk, compilation);
    }

    true
  }

  pub fn update_cache(&mut self, compilation: &Compilation) {
    let chunk_graph = &compilation.chunk_graph;

    for cgi in self.chunk_group_infos.values() {
      let cg = compilation.chunk_group_by_ukey.expect_get(&cgi.chunk_group);
      let chunk = cg.chunks[0];

      let Some(blocks) = self.blocks_by_cgi.get(&cgi.ukey) else {
        continue;
      };

      let module_graph = compilation.get_module_graph();
      for parent in &cg.parents {
        let parent_cgi = self
          .chunk_group_info_map
          .get(parent)
          .expect("should have cgi");
        for block_id in blocks {
          let Some(async_block_id) = block_id.as_async() else {
            continue;
          };
          let block_options = module_graph.block_by_id_expect(&async_block_id);

          self.chunk_caches.insert(
            (
              *parent_cgi,
              DependenciesBlockIdentifier::AsyncDependenciesBlock(async_block_id),
            ),
            ChunkCreateData {
              available_modules: cgi.min_available_modules.clone(),
              options: block_options.get_group_options().cloned(),
              runtime: cgi.runtime.clone(),
              can_rebuild: cg.parents.len() == 1,
              module: block_id.get_root_block(compilation),
              modules: chunk_graph
                .get_chunk_modules(&chunk, &module_graph)
                .into_iter()
                .map(|m| m.identifier())
                .collect(),
              skipped_modules: cgi.skipped_items.clone(),
              outgoings: cgi.outgoing_blocks.clone(),
            },
          );
        }
      }
    }
  }

  pub fn hit_cache(
    &self,
    cache: &ChunkCreateData,
    runtime: &RuntimeSpec,
    new_available_modules: BigUint,
    options: Option<&GroupOptions>,
  ) -> bool {
    cache.can_rebuild
      && !self.available_modules_affected(cache, new_available_modules)
      && options == cache.options.as_ref()
      && is_runtime_equal(runtime, &cache.runtime)
  }

  pub fn available_modules_affected(
    &self,
    cache: &ChunkCreateData,
    new_available_modules: BigUint,
  ) -> bool {
    if new_available_modules == cache.available_modules {
      return false;
    }

    // get changed modules
    // 0010
    // 0100
    // diff: 0110
    let diff = &cache.available_modules ^ new_available_modules;

    for m in cache.modules.iter().chain(cache.skipped_modules.iter()) {
      let m = self.get_module_ordinal(*m);
      if diff.bit(m) {
        return true;
      }
    }

    false
  }
}

#[derive(Debug, Clone)]
pub struct ChunkCreateData {
  // input
  available_modules: BigUint,
  options: Option<GroupOptions>,
  runtime: RuntimeSpec,

  // safe to rebuild from cache, currently only if chunk has unique single parent can be safe to rebuild from cache
  can_rebuild: bool,

  // result
  module: ModuleIdentifier,
  modules: Vec<ModuleIdentifier>,
  skipped_modules: IdentifierIndexSet,
  outgoings: Vec<AsyncDependenciesBlockIdentifier>,
}
