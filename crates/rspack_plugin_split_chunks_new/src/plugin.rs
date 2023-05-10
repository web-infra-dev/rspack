use std::{borrow::Cow, fmt::Debug};

use async_scoped::TokioScope;
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

  async fn inner_impl(&self, compilation: &mut Compilation) {
    let mut module_group_map = self.prepare_module_group_map(compilation).await;

    self.ensure_min_size_fit(compilation, &mut module_group_map);

    while !module_group_map.is_empty() {
      let (_module_group_key, mut module_group) =
        self.find_best_module_group(&mut module_group_map);

      let cache_group = &self.cache_groups[module_group.cache_group_index];

      let mut is_reuse_existing_chunk = false;
      let mut is_reuse_existing_chunk_with_all_modules = false;
      let new_chunk = self.get_corresponding_chunk(
        compilation,
        &mut module_group,
        &mut is_reuse_existing_chunk,
        &mut is_reuse_existing_chunk_with_all_modules,
      );

      let new_chunk_mut = new_chunk.as_mut(&mut compilation.chunk_by_ukey);

      new_chunk_mut
        .chunk_reasons
        .push(["(cache group: ", cache_group.key.as_str(), ")"].join(""));

      new_chunk_mut
        .id_name_hints
        .insert(cache_group.id_hint.clone());

      if is_reuse_existing_chunk {
        // The chunk is not new but created in code splitting. We need remove `new_chunk` since we would remove
        // modules in this `Chunk/ModuleGroup` from other chunks. Other chunks is stored in `ModuleGroup.chunks`.
        module_group.chunks.remove(&new_chunk);
      }

      let used_chunks = Cow::Borrowed(&module_group.chunks);

      self.move_modules_to_new_chunk_and_remove_from_old_chunks(
        &module_group,
        new_chunk,
        &used_chunks,
        compilation,
      );

      self.split_from_original_chunks(&module_group, &used_chunks, new_chunk, compilation);

      self.remove_all_modules_from_other_module_groups(
        &module_group,
        &mut module_group_map,
        &used_chunks,
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
        // Fast path
        if cache_group.min_size.is_empty() {
          return None;
        }

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

  /// Affected by `splitChunks.cacheGroups.{cacheGroup}.reuseExistingChunk`
  ///
  /// If the current chunk contains modules already split out from the main bundle,
  /// it will be reused instead of a new one being generated. This can affect the
  /// resulting file name of the chunk.
  ///
  /// the best means the reused chunks contains all modules in this ModuleGroup
  fn find_the_best_reusable_chunk(
    &self,
    compilation: &mut Compilation,
    module_group: &mut ModuleGroup,
  ) -> Option<ChunkUkey> {
    let candidates = module_group.chunks.par_iter().filter_map(|chunk| {
      let chunk = chunk.as_ref(&compilation.chunk_by_ukey);

      if compilation
        .chunk_graph
        .get_number_of_chunk_modules(&chunk.ukey)
        != module_group.modules.len()
      {
        // Fast path for checking is the chunk reuseable for this `ModuleGroup`.
        return None;
      }

      if module_group.chunks.len() > 1
        && compilation
          .chunk_graph
          .get_number_of_entry_modules(&chunk.ukey)
          > 0
      {
        // `module_group.chunks.len() > 1`: this ModuleGroup are related multiple chunks generated in code splitting.
        // `get_number_of_entry_modules(&chunk.ukey) > 0`:  current chunk is an initial chunk.

        // I(hyf0) don't see why breaking for this condition. But ChatGPT3.5 told me:

        // The condition means that if there are multiple chunks in item and the current chunk is an
        // entry chunk, then it cannot be reused. This is because entry chunks typically contain the core
        // code of an application, while other chunks contain various parts of the application. If
        // an entry chunk is used for other purposes, it may cause the application broken.
        return None;
      }

      let is_all_module_in_chunk = module_group.modules.par_iter().all(|each_module| {
        compilation
          .chunk_graph
          .is_module_in_chunk(each_module, chunk.ukey)
      });
      if !is_all_module_in_chunk {
        return None;
      }

      Some(chunk)
    });

    /// Port https://github.com/webpack/webpack/blob/b471a6bfb71020f6d8f136ef10b7efb239ef5bbf/lib/optimize/SplitChunksPlugin.js#L1360-L1373
    fn best_reuseable_chunk<'a>(first: &'a Chunk, second: &'a Chunk) -> &'a Chunk {
      match (&first.name, &second.name) {
        (None, None) => first,
        (None, Some(_)) => second,
        (Some(_), None) => first,
        (Some(first_name), Some(second_name)) => match first_name.len().cmp(&second_name.len()) {
          std::cmp::Ordering::Greater => second,
          std::cmp::Ordering::Less => first,
          std::cmp::Ordering::Equal => {
            if matches!(second_name.cmp(first_name), std::cmp::Ordering::Less) {
              second
            } else {
              first
            }
          }
        },
      }
    }

    let best_reuseable_chunk =
      candidates.reduce_with(|best, each| best_reuseable_chunk(best, each));

    best_reuseable_chunk.map(|c| c.ukey)
  }

  fn get_corresponding_chunk(
    &self,
    compilation: &mut Compilation,
    module_group: &mut ModuleGroup,
    is_reuse_existing_chunk: &mut bool,
    is_reuse_existing_chunk_with_all_modules: &mut bool,
  ) -> ChunkUkey {
    if let Some(chunk_name) = &module_group.chunk_name {
      if let Some(chunk) = compilation.named_chunks.get(chunk_name) {
        *is_reuse_existing_chunk = true;
        *chunk
      } else {
        let new_chunk = Compilation::add_named_chunk(
          chunk_name.clone(),
          &mut compilation.chunk_by_ukey,
          &mut compilation.named_chunks,
        );
        new_chunk
          .chunk_reasons
          .push("Create by split chunks".to_string());
        compilation.chunk_graph.add_chunk(new_chunk.ukey);
        new_chunk.ukey
      }
    } else if let Some(reusable_chunk) = self.find_the_best_reusable_chunk(compilation, module_group)
      && module_group.cache_group_reuse_existing_chunk
    {
      *is_reuse_existing_chunk = true;
      *is_reuse_existing_chunk_with_all_modules = true;
      reusable_chunk
    } else {
      let new_chunk = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
      new_chunk
        .chunk_reasons
        .push("Create by split chunks".to_string());
      compilation.chunk_graph.add_chunk(new_chunk.ukey);
      new_chunk.ukey
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
      .filter(|(_key, each_module_group)| {
        // Fast path: check whether has overlap on chunks
        each_module_group
          .chunks
          .intersection(used_chunks)
          .next()
          .is_some()
      })
      .filter_map(|(key, each_module_group)| {
        item.modules.iter().for_each(|module| {
          if each_module_group.modules.contains(module) {
            let module = compilation
              .module_graph
              .module_by_identifier(module)
              .unwrap_or_else(|| panic!("Module({module}) not found"));
            each_module_group.remove_module(module);
          }
        });

        if each_module_group.modules.is_empty() {
          Some(key.clone())
        } else {
          None
        }
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

  async fn prepare_module_group_map(&self, compilation: &mut Compilation) -> ModuleGroupMap {
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

    let module_group_map: DashMap<String, ModuleGroup> = DashMap::default();

    async_scoped::Scope::scope_and_block(|scope: &mut TokioScope<'_, _>| {
      for module in compilation.module_graph.modules().values() {
        let module = &**module;

        let belong_to_chunks = compilation
          .chunk_graph
          .get_module_chunks((*module).identifier());

        let module_group_map = &module_group_map;

        for (cache_group_index, cache_group) in self.cache_groups.iter().enumerate() {
          scope.spawn(async move {
            // Filter by `splitChunks.cacheGroups.{cacheGroup}.test`
            let is_match_the_test: bool = (cache_group.test)(module);

            if !is_match_the_test {
              return;
            }

            let selected_chunks = belong_to_chunks
              .iter()
              .map(|c| chunk_db.get(c).expect("Should have a chunk here"))
              // Filter by `splitChunks.cacheGroups.{cacheGroup}.chunks`
              .filter(|c| (cache_group.chunk_filter)(c, chunk_group_db))
              .collect::<Box<[_]>>();

            // Filter by `splitChunks.cacheGroups.{cacheGroup}.minChunks`
            if selected_chunks.len() < cache_group.min_chunks as usize {
              return;
            }

            merge_matched_item_into_module_group_map(
              MatchedItem {
                module,
                cache_group,
                cache_group_index,
                selected_chunks,
              },
              module_group_map,
            )
            .await;

            async fn merge_matched_item_into_module_group_map(
              matched_item: MatchedItem<'_>,
              module_group_map: &DashMap<String, ModuleGroup>,
            ) {
              let MatchedItem {
                module,
                cache_group_index,
                cache_group,
                selected_chunks,
              } = matched_item;

              // `Module`s with the same chunk_name would be merged togother.
              // `Module`s could be in different `ModuleGroup`s.
              let chunk_name: Option<String> = (cache_group.name)(module).await;

              let key: String = if let Some(cache_group_name) = &chunk_name {
                [&cache_group.key, " name:", cache_group_name].join("")
              } else {
                [&cache_group.key, " index:", &cache_group_index.to_string()].join("")
              };

              let mut module_group = module_group_map.entry(key).or_insert_with(|| ModuleGroup {
                modules: Default::default(),
                cache_group_index,
                cache_group_priority: cache_group.priority,
                cache_group_reuse_existing_chunk: cache_group.reuse_existing_chunk,
                sizes: Default::default(),
                chunks: Default::default(),
                chunk_name,
              });

              module_group.add_module(module);
              module_group
                .chunks
                .extend(selected_chunks.iter().map(|c| c.ukey))
            }
          });
        }
      }
    });

    module_group_map.into_iter().collect()
  }
}

impl Debug for SplitChunksPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPlugin").finish()
  }
}

#[async_trait::async_trait]
impl Plugin for SplitChunksPlugin {
  async fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    // use std::time::Instant;
    // let start = Instant::now();
    self.inner_impl(args.compilation).await;
    // let duration = start.elapsed();
    // println!("SplitChunksPlugin is: {:?}", duration);
    Ok(())
  }
}
