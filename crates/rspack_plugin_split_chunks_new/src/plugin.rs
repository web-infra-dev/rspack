use std::fmt::Debug;

use dashmap::DashMap;
use rayon::prelude::*;
use rspack_core::{ChunkUkey, Compilation, Module, Plugin};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  cache_group::CacheGroup,
  module_group::{compare_entries, ModuleGroup},
};

struct _OverallOptions {
  pub cache_groups: Vec<CacheGroup>,
  pub min_chunks: u32,
  pub max_size: f64,
  pub min_size: f64,
}

type ModuleGroupMap = FxHashMap<String, ModuleGroup>;

pub struct PluginOptions {
  pub cache_groups: Vec<CacheGroup>,
}

pub struct SplitChunksPlugin {
  // overall: OverallOptions,
  cache_groups: Vec<CacheGroup>,
}

impl SplitChunksPlugin {
  pub fn new(options: PluginOptions) -> Self {
    Self {
      cache_groups: options.cache_groups,
    }
  }

  fn inner_impl(&self, compilation: &mut Compilation) {
    let mut module_group_map = self.prepare_module_and_chunks_info_map(compilation);

    while !module_group_map.is_empty() {
      let (_module_group_key, module_group) = self.find_best_module_group(&mut module_group_map);

      let new_chunk = if let Some(chunk) = compilation.named_chunks.get(&module_group.name) {
        *chunk
      } else {
        let chunk = Compilation::add_named_chunk(
          module_group.name.clone(),
          &mut compilation.chunk_by_ukey,
          &mut compilation.named_chunks,
        );

        chunk
          .chunk_reasons
          .push("Create by split chunks".to_string());
        chunk.ukey
      };
      compilation.chunk_graph.add_chunk(new_chunk);

      let used_chunks = &module_group.chunks;
      self.move_modules_to_new_chunk_and_remove_from_old_chunks(
        &module_group,
        new_chunk,
        used_chunks,
        compilation,
      );
      self.split_from_original_chunks(&module_group, used_chunks, new_chunk, compilation);

      self.remove_all_modules_from_other_module_groups(
        &module_group,
        &mut module_group_map,
        used_chunks,
        compilation,
      )
    }
  }

  fn _ensure_max_size_fit(&self, _compilation: &mut Compilation) {}

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
    used_chunks: &FxHashSet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    for module_identifier in &item.modules {
      // First, we remove modules from old chunks

      // Remove module from old chunks
      for used_chunk in used_chunks {
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
      let [new_chunk, original_chunk] = compilation
        .chunk_by_ukey
        ._todo_should_remove_this_method_inner_mut()
        .get_many_mut([&new_chunk_ukey, original_chunk])
        .expect("TODO:");
      original_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);
    }
  }

  fn find_best_module_group(&self, module_group_map: &mut ModuleGroupMap) -> (String, ModuleGroup) {
    // perf(hyf): I wonder if we could use BinaryHeap to avoid sorting for find_best_module_group call
    debug_assert!(!module_group_map.is_empty());
    let mut iter = module_group_map.iter();
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

  fn prepare_module_and_chunks_info_map(&self, compilation: &mut Compilation) -> ModuleGroupMap {
    let chunk_db = &compilation.chunk_by_ukey;
    let chunk_group_db = &compilation.chunk_group_by_ukey;

    let module_and_corresponding_cache_group = compilation
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
            let is_match_the_test = (cache_group.test)(module);

            if !is_match_the_test {
              return None;
            }

            let selected_chunks = belong_to_chunks
              .iter()
              .map(|c| chunk_db.get(c).expect("Should have a chunk here"))
              .filter(|c| (cache_group.chunk_filter)(c, chunk_group_db))
              .collect::<Vec<_>>();

            if selected_chunks.len() < cache_group.min_chunks as usize {
              return None;
            }

            Some((module, cache_group_index, cache_group, selected_chunks))
          },
        )
      });

    let chunks_info_map: DashMap<String, ModuleGroup> = DashMap::default();

    module_and_corresponding_cache_group.for_each(
      |(module, cache_group_index, cache_group, selected_chunks)| {
        let key = ["name: ", &cache_group.name].join("");

        let mut chunks_info_item = chunks_info_map.entry(key).or_insert_with(|| ModuleGroup {
          // The ChunkInfoItem is not existed. Initialize it.
          modules: Default::default(),
          cache_group_index,
          cache_group_priority: cache_group.priority,
          sizes: Default::default(),
          chunks: Default::default(),
          name: cache_group.name.clone(),
        });

        chunks_info_item.add_module(module);
        chunks_info_item
          .chunks
          .extend(selected_chunks.iter().map(|c| c.ukey))
      },
    );

    chunks_info_map.into_iter().collect()
  }
}

trait EstimatedSize {
  fn estimated_size(&self, source_type: &rspack_core::SourceType) -> f64;
}

impl<T: Module> EstimatedSize for T {
  fn estimated_size(&self, source_type: &rspack_core::SourceType) -> f64 {
    use rspack_core::ModuleType;
    let coefficient: f64 = match self.module_type() {
      // 5.0 is a number in practice
      rspack_core::ModuleType::Jsx
      | ModuleType::JsxDynamic
      | ModuleType::JsxEsm
      | ModuleType::Tsx => 7.5,
      ModuleType::Js | ModuleType::JsDynamic => 1.5,
      _ => 1.0,
    };

    self.size(source_type) * coefficient
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
