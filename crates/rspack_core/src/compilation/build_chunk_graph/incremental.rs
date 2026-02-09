use std::{collections::HashSet, hash::BuildHasherDefault, sync::Arc};

use num_bigint::BigUint;
use rspack_collections::{
  IdentifierHasher, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeySet,
};
use rspack_error::Result;
use tracing::instrument;

use super::code_splitter::{CgiUkey, CodeSplitter, DependenciesBlockIdentifier};
use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGroupKind, ChunkGroupUkey, ChunkUkey, Compilation,
  GroupOptions, ModuleIdentifier, RuntimeSpec,
  incremental::{IncrementalPasses, Mutation},
  is_runtime_equal,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChunkReCreation {
  Entry {
    name: String,
    depend_on: Option<Vec<String>>,
  },
  Normal(NormalChunkRecreation),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NormalChunkRecreation {
  pub block: AsyncDependenciesBlockIdentifier,
  module: ModuleIdentifier,
  pub cgi: CgiUkey,
  chunk: ChunkUkey,
}

impl ChunkReCreation {
  pub fn rebuild(self, splitter: &mut CodeSplitter, compilation: &mut Compilation) -> Result<()> {
    match self {
      ChunkReCreation::Entry {
        name: entry,
        depend_on: _,
      } => {
        let input = splitter.prepare_entry_input(&entry, compilation)?;
        splitter.set_entry_runtime_and_depend_on(&entry, compilation)?;
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
    let chunk_graph = &mut compilation.build_chunk_graph_artifact.chunk_graph;

    // Step 1. find all invalidate chunk groups and remove module from ChunkGraph
    let Some(cgm) = chunk_graph.get_chunk_graph_module_mut(module) else {
      return Ok(vec![]);
    };

    let invalidate_chunk_groups = cgm
      .chunks
      .iter()
      .flat_map(|chunk| {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk);
        chunk.groups().clone()
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

        if let Some(mut mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkRemove { chunk: *chunk_ukey });
        }
      }
    }

    // remove chunk group
    compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .remove(&chunk_group_ukey);

    let mut edges = vec![];

    // use `if let` because entry don't have incoming_blocks_by_cgi
    if let Some(incoming_blocks) = self.incoming_blocks_by_cgi.get(&chunk_group_info.ukey) {
      for block in incoming_blocks {
        self.block_to_chunk_group.remove(block);

        let Some(block) = block.as_async() else {
          continue;
        };

        let Some(cache) = self.chunk_caches.get(&block) else {
          continue;
        };

        let Some(parents) = self.block_owner.get(&block) else {
          continue;
        };

        for parent_cgi in parents {
          let Some(parent_cg) = self.chunk_group_infos.get(parent_cgi) else {
            continue;
          };

          let parent_group = compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .expect_get(&parent_cg.chunk_group);

          edges.push(ChunkReCreation::Normal(NormalChunkRecreation {
            block,
            module: cache.module,
            cgi: *parent_cgi,
            chunk: parent_group.chunks[0],
          }));
        }
      }
    };

    self.stat_invalidated_chunk_group += 1;

    for child in chunk_group_info.children.iter() {
      let Some(child_cgi) = self.chunk_group_infos.get_mut(child) else {
        continue;
      };

      let cg = child_cgi.chunk_group;

      self.invalidate_chunk_group(cg, compilation)?;
    }

    if let Some(name) = chunk_group_name
      && chunk_group.is_initial()
      && chunk_group.parents.is_empty()
    {
      edges = vec![ChunkReCreation::Entry {
        name,
        depend_on: None,
      }];
    }

    // If the invalidated chunk group is an entrypoint, we must also invalidate any
    // other entrypoints that depend on it via the `dependOn` option.
    if matches!(chunk_group.kind, ChunkGroupKind::Entrypoint { .. }) {
      let dependent_entrypoints_to_invalidate: Vec<_> = compilation
        .build_chunk_graph_artifact
        .entrypoints
        .values()
        .filter_map(|entrypoint_ukey| {
          compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .get(entrypoint_ukey)
            .filter(|entrypoint| entrypoint.parents.contains(&chunk_group_ukey))
            .map(|_| *entrypoint_ukey)
        })
        .collect();

      for entrypoint_ukey in dependent_entrypoints_to_invalidate {
        if let Some(invalidation_result) =
          self.invalidate_chunk_group(entrypoint_ukey, compilation)?
        {
          edges.extend(invalidation_result);
        }
      }
    }

    if edges.is_empty() {
      return Ok(None);
    }

    Ok(Some(edges))
  }

  fn collect_dirty_caches(
    &self,
    compilation: &Compilation,
    modules: impl Iterator<Item = ModuleIdentifier>,
  ) -> HashSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>> {
    let chunk_graph: &crate::ChunkGraph = &compilation.build_chunk_graph_artifact.chunk_graph;
    let mut chunk_groups = UkeySet::default();
    let mut removed: HashSet<
      AsyncDependenciesBlockIdentifier,
      BuildHasherDefault<IdentifierHasher>,
    > = Default::default();
    for m in modules {
      let Some(cgm) = chunk_graph.chunk_graph_module_by_module_identifier.get(&m) else {
        continue;
      };

      for chunk_ukey in &cgm.chunks {
        let chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey);
        chunk_groups.extend(chunk.groups().clone());
      }
    }
    for group in chunk_groups {
      let cgi = self
        .chunk_group_info_map
        .get(&group)
        .expect("should have chunk group");
      let Some(blocks) = self.incoming_blocks_by_cgi.get(cgi).cloned() else {
        continue;
      };

      for block in blocks {
        if let DependenciesBlockIdentifier::AsyncDependenciesBlock(async_block) = block {
          removed.insert(async_block);
        }
      }
    }

    removed
  }

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
      self.invalidate_chunk_group(*removed_cg, compilation)?;
    }

    if !removed.is_empty() {
      self.remove_orphan(compilation)?;
    }

    Ok(())
  }

  pub fn recover_from_cache(&mut self, cgi_ukey: CgiUkey, compilation: &mut Compilation) -> bool {
    if !compilation
      .incremental
      .mutations_readable(IncrementalPasses::BUILD_CHUNK_GRAPH)
    {
      return false;
    }

    let Some(cgi) = self.chunk_group_infos.get(&cgi_ukey) else {
      return false;
    };
    let Some(blocks) = self.incoming_blocks_by_cgi.get(&cgi.ukey) else {
      return false;
    };

    let cg = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(&cgi.chunk_group);
    let chunk = cg.chunks[0];

    let Some(cache) = &blocks
      .iter()
      .next()
      .expect("should have one block")
      .as_async()
      .and_then(|block_id| self.chunk_caches.get(&block_id).cloned())
    else {
      return false;
    };

    if !cache.can_rebuild {
      self.stat_cache_miss_by_cant_rebuild += 1;
      return false;
    }

    let module_graph = compilation.get_module_graph();
    let DependenciesBlockIdentifier::AsyncDependenciesBlock(block_id) =
      blocks.iter().next().expect("should have one block")
    else {
      return false;
    };

    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");

    if !cgi.min_available_modules_init
      || !self.hit_cache(
        cache,
        &cgi.runtime,
        cgi.min_available_modules.clone(),
        block.get_group_options(),
      )
    {
      self.stat_cache_miss_by_available_modules += 1;
      return false;
    }

    let cache_result = cache
      .cache_result
      .as_ref()
      .expect("should have cache result");

    // update cache available modules
    self.outdated_chunk_group_info.insert(cgi_ukey);

    let cgi = self.chunk_group_infos.expect_get_mut(&cgi_ukey);
    let group_ukey = cgi.chunk_group;
    cgi.skipped_items = cache_result.skipped_modules.clone();

    let chunk_graph = &mut compilation.build_chunk_graph_artifact.chunk_graph;
    for module in &cache_result.modules {
      let ordinal = self.get_module_ordinal(*module);
      chunk_graph.connect_chunk_and_module(chunk, *module);

      let mask = self.mask_by_chunk.entry(chunk).or_default();
      mask.set_bit(ordinal, true);
    }

    let group = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get_mut(&group_ukey);
    group.module_pre_order_indices = cache_result.pre_order_indices.clone();
    group.module_post_order_indices = cache_result.post_order_indices.clone();

    for block in &cache_result.outgoings {
      self.make_chunk_group(
        *block,
        *self.edges.get(block).expect("should have module for block"),
        cgi_ukey,
        chunk,
        compilation,
      );
    }

    true
  }

  #[instrument(skip_all)]
  pub fn update_with_compilation(&mut self, compilation: &mut Compilation) -> Result<()> {
    // TODO: heuristic incremental update is temporarily disabled
    // Original code:
    // let enable_incremental = compilation
    //   .incremental
    //   .mutations_readable(IncrementalPasses::BUILD_CHUNK_GRAPH);
    let enable_incremental = false;

    // This optimization is from  https://github.com/webpack/webpack/pull/18090 by https://github.com/dmichon-msft
    // Thanks!
    let module_graph = compilation.get_module_graph();
    let ordinal_by_module = &mut self.ordinal_by_module;
    for (m, _) in module_graph.modules() {
      if !ordinal_by_module.contains_key(&m) {
        ordinal_by_module.insert(m, ordinal_by_module.len() as u64 + 1);
      }
    }
    for chunk in compilation.build_chunk_graph_artifact.chunk_by_ukey.keys() {
      let mut mask = BigUint::from(0u32);
      for module_id in compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier(chunk)
      {
        let module_ordinal = self.get_module_ordinal(*module_id);
        mask.set_bit(module_ordinal, true);
      }
      self.mask_by_chunk.insert(*chunk, mask);
    }

    self.stat_invalidated_chunk_group = 0;
    self.stat_invalidated_caches = 0;
    self.stat_use_cache = 0;
    self.stat_chunk_group_created = 0;
    self.stat_cache_miss_by_available_modules = 0;
    self.stat_cache_miss_by_cant_rebuild = 0;

    if !enable_incremental {
      return Ok(());
    }

    let (affected_modules, removed_modules) = if let Some(mutations) = compilation
      .incremental
      .mutations_read(IncrementalPasses::BUILD_CHUNK_GRAPH)
    {
      let affected_modules = mutations.get_affected_modules_with_module_graph(module_graph);
      let removed_modules: IdentifierSet = mutations
        .iter()
        .filter_map(|mutation| match mutation {
          Mutation::ModuleRemove { module } => Some(*module),
          _ => None,
        })
        .collect();
      (affected_modules, removed_modules)
    } else {
      (
        compilation
          .get_module_graph()
          .modules()
          .map(|(module_identifier, _)| module_identifier)
          .collect(),
        Default::default(),
      )
    };

    let mut edges = vec![];

    // collect invalidate caches before we do anything to the chunk graph
    let dirty_blocks = self.collect_dirty_caches(
      compilation,
      affected_modules
        .iter()
        .chain(removed_modules.iter())
        .copied(),
    );

    let mut removed_entries = UkeySet::default();
    for (name, chunk_group) in compilation.entrypoints() {
      if !compilation.entries.contains_key(name) {
        removed_entries.insert(*chunk_group);
      }
    }

    for m in affected_modules {
      for module_map in self.block_modules_runtime_map.values_mut() {
        module_map.remove(&DependenciesBlockIdentifier::Module(m));
      }

      let more_edges = self.invalidate_from_module(m, compilation)?;
      edges.extend(more_edges);
    }

    if !removed_modules.is_empty() {
      // TODO:
      // if we have removed module, we should invalidate its
      // incomings, but we cannot get its incomings at present,
      self.block_modules_runtime_map.clear();

      for m in removed_modules {
        self.invalidate_from_module(m, compilation)?;
      }
    }

    self.stat_invalidated_caches = dirty_blocks.len() as u32;
    for block in dirty_blocks {
      self.chunk_caches.remove(&block);
    }

    // remove async entrypoints
    compilation
      .build_chunk_graph_artifact
      .async_entrypoints
      .retain(|cg_ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .contains(cg_ukey)
      });

    // If after edges rebuild there are still some entries not included in entrypoints
    // then they are new added entries and we build them.
    let module_graph = compilation.get_module_graph();

    if !removed_entries.is_empty() {
      // If there is removed entry, we need to rebuild all entries
      let entrypoints = compilation
        .build_chunk_graph_artifact
        .entrypoints
        .values()
        .copied()
        .collect::<Vec<_>>();
      for entry in entrypoints {
        self.invalidate_chunk_group(entry, compilation)?;
      }

      edges = compilation
        .entries
        .iter()
        .map(|(name, entry_data)| ChunkReCreation::Entry {
          name: name.to_owned(),
          depend_on: entry_data.options.depend_on.clone(),
        })
        .collect();
    } else {
      'outer: for (entry, data) in &compilation.entries {
        if !compilation
          .build_chunk_graph_artifact
          .entrypoints
          .contains_key(entry)
        {
          // new entry
          edges.push(ChunkReCreation::Entry {
            name: entry.to_owned(),
            depend_on: data.options.depend_on.clone(),
          });
          continue 'outer;
        }

        let curr_modules = data
          .all_dependencies()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
          .copied()
          .collect::<IdentifierSet>();

        let curr_entry_chunk_ukey = compilation.entrypoint_by_name(entry).chunks[0];
        let prev_entry_modules = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_entry_modules(&curr_entry_chunk_ukey)
          .into_iter()
          .collect::<IdentifierSet>();

        for m in &curr_modules {
          // there is new entry
          if !prev_entry_modules.contains(m) {
            edges.push(ChunkReCreation::Entry {
              name: entry.to_owned(),
              depend_on: data.options.depend_on.clone(),
            });
            continue 'outer;
          }
        }

        for m in prev_entry_modules {
          if !curr_modules.contains(&m) {
            // there is removed entry modules
            edges.push(ChunkReCreation::Entry {
              name: entry.to_owned(),
              depend_on: data.options.depend_on.clone(),
            });
            continue 'outer;
          }
        }
      }
    }

    // For entries with dependencies (`dependOn`), we must ensure that the dependency
    // is processed before the entry that depends on it. This is a topological sort.
    edges.sort_unstable_by(|a, b| {
      let (a_name, a_depend_on) = if let ChunkReCreation::Entry { name, depend_on } = a {
        (name, depend_on)
      } else {
        return std::cmp::Ordering::Equal;
      };

      let (b_name, b_depend_on) = if let ChunkReCreation::Entry { name, depend_on } = b {
        (name, depend_on)
      } else {
        return std::cmp::Ordering::Equal;
      };

      let a_depends_on_b = a_depend_on
        .as_deref()
        .is_some_and(|deps| deps.contains(b_name));
      let b_depends_on_a = b_depend_on
        .as_deref()
        .is_some_and(|deps| deps.contains(a_name));

      // If a depends on b, b must come first (a > b).
      // If b depends on a, a must come first (a < b).
      // If there's no direct dependency, their relative order doesn't matter.
      if a_depends_on_b {
        std::cmp::Ordering::Greater
      } else if b_depends_on_a {
        std::cmp::Ordering::Less
      } else {
        std::cmp::Ordering::Equal
      }
    });
    for edge in edges {
      edge.rebuild(self, compilation)?;
    }

    // Ensure entrypoints always have the same order with entries
    compilation
      .build_chunk_graph_artifact
      .entrypoints
      .sort_unstable_by(|a, _, b, _| {
        let a = compilation
          .entries
          .get_index_of(a)
          .expect("entrypoints must exist in entries");
        let b = compilation
          .entries
          .get_index_of(b)
          .expect("entrypoints must exist in entries");
        a.cmp(&b)
      });

    Ok(())
  }

  pub fn update_cache(&mut self, compilation: &Compilation) {
    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;

    self.chunk_caches.clear();

    for cgi in self.chunk_group_infos.values() {
      let cg = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(&cgi.chunk_group);
      let chunk = cg.chunks[0];
      let module_graph = compilation.get_module_graph();

      let Some(blocks) = self.incoming_blocks_by_cgi.get(&cgi.ukey) else {
        continue;
      };

      for block_id in blocks {
        let DependenciesBlockIdentifier::AsyncDependenciesBlock(block_id) = block_id else {
          continue;
        };
        let block_options = module_graph.block_by_id_expect(block_id);
        let module = *self
          .edges
          .get(block_id)
          .expect("should have module for block_id");

        let can_rebuild = cg.parents.len() == 1;

        let group = compilation
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(&cgi.chunk_group);
        self.chunk_caches.insert(
          *block_id,
          ChunkCreateData {
            available_modules: cgi.min_available_modules.clone(),
            options: block_options.get_group_options().cloned(),
            runtime: cgi.runtime.as_ref().clone(),
            can_rebuild,
            module,
            cache_result: can_rebuild.then(|| CacheResult {
              modules: chunk_graph
                .get_chunk_modules_identifier(&chunk)
                .iter()
                .copied()
                .collect::<Vec<_>>(),
              pre_order_indices: group.module_pre_order_indices.clone(),
              post_order_indices: group.module_post_order_indices.clone(),
              skipped_modules: cgi.skipped_items.clone(),
              outgoings: cgi.outgoing_blocks.clone(),
            }),
          },
        );
      }
    }
  }

  pub fn hit_cache(
    &self,
    cache: &ChunkCreateData,
    runtime: &RuntimeSpec,
    new_available_modules: Arc<BigUint>,
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
    new_available_modules: Arc<BigUint>,
  ) -> bool {
    if new_available_modules == cache.available_modules {
      return false;
    }

    // get changed modules
    // 0010
    // 0100
    // diff: 0110
    let diff = cache.available_modules.as_ref() ^ new_available_modules.as_ref();

    let cache_result = cache
      .cache_result
      .as_ref()
      .expect("should have cache result");

    for m in cache_result
      .modules
      .iter()
      .chain(cache_result.skipped_modules.iter())
    {
      let m = self.get_module_ordinal(*m);
      if diff.bit(m) {
        return true;
      }
    }

    false
  }
}

#[derive(Debug, Clone)]
struct CacheResult {
  pub modules: Vec<ModuleIdentifier>,
  pub pre_order_indices: IdentifierMap<usize>,
  pub post_order_indices: IdentifierMap<usize>,
  pub skipped_modules: IdentifierIndexSet,
  pub outgoings: std::collections::HashSet<
    AsyncDependenciesBlockIdentifier,
    BuildHasherDefault<IdentifierHasher>,
  >,
}

#[derive(Debug, Clone)]
pub struct ChunkCreateData {
  // input
  available_modules: Arc<BigUint>,
  options: Option<GroupOptions>,
  runtime: RuntimeSpec,
  pub module: ModuleIdentifier,

  // safe to rebuild from cache, currently only if chunk has unique single parent can be safe to rebuild from cache
  can_rebuild: bool,

  // result
  cache_result: Option<CacheResult>,
}
