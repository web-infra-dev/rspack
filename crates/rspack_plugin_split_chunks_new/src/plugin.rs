use std::fmt::Debug;

use dashmap::DashMap;
use rayon::prelude::*;
use rspack_core::{Chunk, ChunkUkey, Compilation, Module, Plugin, SourceType};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  cache_group::CacheGroup,
  module_group::{compare_entries, ModuleGroup},
};

type ModuleGroupMap = FxHashMap<String, ModuleGroup>;

pub struct PluginOptions {
  pub cache_groups: Vec<CacheGroup>,
}

pub struct SplitChunksPlugin {
  cache_groups: Box<[CacheGroup]>,
}

impl SplitChunksPlugin {
  pub fn new(options: PluginOptions) -> Self {
    Self {
      cache_groups: options.cache_groups.into(),
    }
  }

  fn inner_impl(&self, compilation: &mut Compilation) {
    let mut module_group_map = self.prepare_module_group_map(compilation);

    self.ensure_min_size_fit(compilation, &mut module_group_map);

    while !module_group_map.is_empty() {
      let (_module_group_key, module_group) = self.find_best_module_group(&mut module_group_map);

      let new_chunk = self.get_corresponding_chunk(compilation, &module_group);

      let original_chunks = &module_group.chunks;

      self.move_modules_to_new_chunk_and_remove_from_old_chunks(
        &module_group,
        new_chunk,
        original_chunks,
        compilation,
      );

      self.split_from_original_chunks(&module_group, original_chunks, new_chunk, compilation);

      self.remove_all_modules_from_other_module_groups(
        &module_group,
        &mut module_group_map,
        original_chunks,
        compilation,
      )
    }
  }

  /// Affected by `splitChunks.minSize`/`splitChunks.cacheGroups.{cacheGroup}.minSize`
  fn ensure_min_size_fit(&self, compilation: &Compilation, module_group_map: &mut ModuleGroupMap) {
    let invalidated_module_groups = module_group_map
      .par_iter_mut()
      .filter_map(|(module_group_key, module_group)| {
        let cache_group = &self.cache_groups[module_group.cache_group_index];

        // Find out what `SourceType`'s size is not fit the min_size
        let violating_source_types: Box<[SourceType]> = module_group
          .sizes
          .iter()
          .filter_map(|(module_group_ty, module_group_ty_size)| {
            let cache_group_ty_min_size = cache_group
              .min_size
              .get(module_group_ty)
              .copied()
              .unwrap_or_default();

            if *module_group_ty_size < cache_group_ty_min_size {
              Some(*module_group_ty)
            } else {
              None
            }
          })
          .collect::<Box<[_]>>();

        // Remove modules having violating SourceType
        let violating_modules = module_group
          .modules
          .par_iter()
          .filter_map(|module_id| {
            let module = &**compilation
              .module_graph
              .module_by_identifier(module_id)
              .expect("Should have a module");
            let having_violating_source_type = violating_source_types
              .iter()
              .any(|ty: &SourceType| module.source_types().contains(ty));
            if having_violating_source_type {
              Some(module)
            } else {
              None
            }
          })
          .collect::<Vec<_>>();

        // question: After removing violating modules, the size of other `SourceType`s of this `ModuleGroup`
        // may not fit again. But Webpack seems ignore this case. Not sure if it is on purpose.
        violating_modules
          .into_iter()
          .for_each(|violating_module| module_group.remove_module(violating_module));

        if module_group.modules.is_empty() {
          Some(module_group_key.clone())
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    invalidated_module_groups.into_iter().for_each(|key| {
      module_group_map.remove(&key);
    });
  }

  fn get_corresponding_chunk(
    &self,
    compilation: &mut Compilation,
    module_group: &ModuleGroup,
  ) -> ChunkUkey {
    if let Some(chunk) = compilation.named_chunks.get(&module_group.chunk_name) {
      *chunk
    } else {
      let chunk = Compilation::add_named_chunk(
        module_group.chunk_name.clone(),
        &mut compilation.chunk_by_ukey,
        &mut compilation.named_chunks,
      );

      chunk
        .chunk_reasons
        .push("Create by split chunks".to_string());
      compilation.chunk_graph.add_chunk(chunk.ukey);
      chunk.ukey
    }
  }

  fn remove_all_modules_from_other_module_groups(
    &self,
    item: &ModuleGroup,
    module_group_map: &mut ModuleGroupMap,
    used_chunks: &FxHashSet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    // remove all modules from other entries and update size
    let keys_of_empty_group = module_group_map
      .iter_mut()
      .par_bridge()
      .filter_map(|(key, each_module_group)| {
        let has_overlap = each_module_group.chunks.union(used_chunks).next().is_some();
        if has_overlap {
          let mut updated = false;
          for module in &item.modules {
            if each_module_group.modules.contains(module) {
              let module = compilation
                .module_graph
                .module_by_identifier(module)
                .unwrap_or_else(|| panic!("Module({module}) not found"));
              each_module_group.remove_module(module);
              updated = true;
            }
          }

          if updated && each_module_group.modules.is_empty() {
            return Some(key.clone());
          }
        }

        None
      })
      .collect::<Vec<_>>();

    keys_of_empty_group.into_iter().for_each(|key| {
      module_group_map.remove(&key);
    });
  }

  /// This de-duplicated each module fro other chunks, make sure there's only one copy of each module.
  fn move_modules_to_new_chunk_and_remove_from_old_chunks(
    &self,
    item: &ModuleGroup,
    new_chunk: ChunkUkey,
    original_chunks: &FxHashSet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    for module_identifier in &item.modules {
      // First, we remove modules from old chunks

      // Remove module from old chunks
      for used_chunk in original_chunks {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(used_chunk, *module_identifier);
      }

      // Add module to new chunk
      compilation
        .chunk_graph
        .connect_chunk_and_module(new_chunk, *module_identifier);
    }
  }

  /// Since the modules are moved into the `new_chunk`, we should
  /// create a connection between the `new_chunk` and `original_chunks`.
  /// Thus, if `original_chunks` want to know which chunk contains moved modules,
  /// it could easily find out.
  fn split_from_original_chunks(
    &self,
    _item: &ModuleGroup,
    original_chunks: &FxHashSet<ChunkUkey>,
    new_chunk: ChunkUkey,
    compilation: &mut Compilation,
  ) {
    let new_chunk_ukey = new_chunk;
    for original_chunk in original_chunks {
      debug_assert!(&new_chunk_ukey != original_chunk);
      let [new_chunk, original_chunk] = compilation
        .chunk_by_ukey
        ._todo_should_remove_this_method_inner_mut()
        .get_many_mut([&new_chunk_ukey, original_chunk])
        .expect("split_from_original_chunks failed");
      original_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);
    }
  }

  fn find_best_module_group(&self, module_group_map: &mut ModuleGroupMap) -> (String, ModuleGroup) {
    // perf(hyf): I wonder if we could use BinaryHeap to avoid sorting for find_best_module_group call
    debug_assert!(!module_group_map.is_empty());
    let mut iter: std::collections::hash_map::Iter<String, ModuleGroup> = module_group_map.iter();
    let (key, mut best_module_group) = iter.next().expect("at least have one item");

    let mut best_entry_key = key.clone();
    for (key, each_module_group) in iter {
      if compare_entries(best_module_group, each_module_group) < 0f64 {
        best_entry_key = key.clone();
        best_module_group = each_module_group;
      }
    }

    let best_module_group = module_group_map
      .remove(&best_entry_key)
      .expect("item should exist");
    (best_entry_key, best_module_group)
  }

  fn prepare_module_group_map(&self, compilation: &mut Compilation) -> ModuleGroupMap {
    let chunk_db = &compilation.chunk_by_ukey;
    let chunk_group_db = &compilation.chunk_group_by_ukey;

    /// If a module meets requirements of a `ModuleGroup`. We consider the `Module` and the `CacheGroup`
    /// to be a `MatchedItem`, which are consumed later to calculate `ModuleGroup`.
    struct MatchedItem<'a> {
      module: &'a dyn Module,
      cache_group_index: usize,
      cache_group: &'a CacheGroup,
      selected_chunks: Box<[&'a Chunk]>,
    }

    let matched_items = compilation
      .module_graph
      .modules()
      .values()
      .par_bridge()
      .flat_map(|module| {
        let belong_to_chunks = compilation
          .chunk_graph
          .get_module_chunks((*module).identifier());

        // A module may match multiple CacheGroups
        self.cache_groups.par_iter().enumerate().filter_map(
          move |(cache_group_index, cache_group)| {
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
            let is_match_the_test: bool = (cache_group.test)(module);

            if !is_match_the_test {
              return None;
            }

            let selected_chunks = belong_to_chunks
              .iter()
              .map(|c| chunk_db.get(c).expect("Should have a chunk here"))
              // Filter by `splitChunks.cacheGroups.{cacheGroup}.chunks`
              .filter(|c| (cache_group.chunk_filter)(c, chunk_group_db))
              .collect::<Box<[_]>>();

            // Filter by `splitChunks.cacheGroups.{cacheGroup}.minChunks`
            if selected_chunks.len() < cache_group.min_chunks as usize {
              return None;
            }

            Some(MatchedItem {
              module: &**module,
              cache_group,
              cache_group_index,
              selected_chunks,
            })
          },
        )
      });

    let module_group_map: DashMap<String, ModuleGroup> = DashMap::default();

    matched_items.for_each(|matched_item| {
      let MatchedItem {
        module,
        cache_group_index,
        cache_group,
        selected_chunks,
      } = matched_item;

      // Merge the `Module` of `MatchedItem` into the `ModuleGroup` which has the same `key`/`cache_group.name`
      let key = ["name: ", &cache_group.name].join("");

      let mut module_group = module_group_map.entry(key).or_insert_with(|| ModuleGroup {
        modules: Default::default(),
        cache_group_index,
        cache_group_priority: cache_group.priority,
        sizes: Default::default(),
        chunks: Default::default(),
        chunk_name: cache_group.name.clone(),
      });

      module_group.add_module(module);
      module_group
        .chunks
        .extend(selected_chunks.iter().map(|c| c.ukey))
    });

    module_group_map.into_iter().collect()
  }
}

impl Debug for SplitChunksPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPlugin").finish()
  }
}

impl Plugin for SplitChunksPlugin {
  fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    self.inner_impl(args.compilation);
    Ok(())
  }
}
