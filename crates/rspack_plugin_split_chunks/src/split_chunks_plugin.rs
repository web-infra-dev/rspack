#![allow(clippy::obfuscated_if_else)]
#![allow(clippy::comparison_chain)]

use std::{fmt::Debug, sync::Arc};

use rspack_core::{Chunk, ChunkGroupByUkey, ChunkUkey, Compilation, Module, Plugin};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  chunks_info_item::ChunksInfoItem,
  max_size_queue_item::MaxSizeQueueItem,
  plugin::{
    create_cache_group, create_cache_group_source, remove_min_size_violating_modules,
    remove_modules_with_source_type,
  },
  utils::{
    check_min_size, check_min_size_reduction, combine_sizes, compare_entries, get_requests,
    get_violating_min_sizes, merge_sizes, merge_sizes2, normalize_chunks_filter, normalize_sizes,
  },
  CacheGroup, CacheGroupSource, ChunkType, ChunksInfoMap, NormalizedFallbackCacheGroup,
  NormalizedOptions, SizeType, SplitChunkSizes, SplitChunksOptions,
};

#[derive(Debug)]
pub struct SplitChunksPlugin {
  raw_options: SplitChunksOptions,
  _cache_group_source_by_key: HashMap<String, CacheGroupSource>,
  cache_group_by_key: HashMap<String, CacheGroup>,
}

impl SplitChunksPlugin {
  pub fn new(options: SplitChunksOptions) -> Self {
    let default_size_types = options
      .default_size_types
      .clone()
      .unwrap_or_else(|| vec![SizeType::JavaScript, SizeType::Unknown]);

    let fallback_cache_group = options.fallback_cache_group.clone().unwrap_or_default();

    let min_size = normalize_sizes(options.min_size, &default_size_types);
    let min_size_reduction = normalize_sizes(options.min_size_reduction, &default_size_types);
    let max_size = normalize_sizes(options.max_size, &default_size_types);

    let normalized_options = NormalizedOptions {
      chunk_filter: {
        let chunks = options.chunks;
        Arc::new(move |chunk, chunk_group_by_ukey| {
          let chunk_type = chunks.as_ref().unwrap_or(&ChunkType::Async);
          chunk_type.is_selected(chunk, chunk_group_by_ukey)
        })
      },
      default_size_types: default_size_types.clone(),
      min_size: min_size.clone(),
      min_size_reduction,
      min_remaining_size: merge_sizes2(
        normalize_sizes(options.min_remaining_size, &default_size_types),
        min_size.clone(),
      ),
      enforce_size_threshold: normalize_sizes(options.enforce_size_threshold, &default_size_types),
      max_async_size: merge_sizes2(
        normalize_sizes(options.max_async_size, &default_size_types),
        max_size.clone(),
      ),
      max_initial_size: merge_sizes2(
        normalize_sizes(options.max_initial_size, &default_size_types),
        max_size,
      ),
      min_chunks: options.min_chunks.unwrap_or(1),
      max_async_requests: options.min_chunks.unwrap_or(1),
      max_initial_requests: options.min_chunks.unwrap_or(1),
      filename: options.filename.clone(),
      get_name: options.name.clone().unwrap_or_else(|| Arc::new(|_| None)),
      fallback_cache_group: NormalizedFallbackCacheGroup {
        chunks_filter: normalize_chunks_filter(
          fallback_cache_group
            .chunks
            .unwrap_or_else(|| options.chunks.unwrap_or(ChunkType::All)),
        ),
        min_size: merge_sizes2(
          normalize_sizes(fallback_cache_group.min_size, &default_size_types),
          min_size,
        ),
        max_async_size: merge_sizes(vec![
          normalize_sizes(fallback_cache_group.max_async_size, &default_size_types),
          normalize_sizes(fallback_cache_group.max_size, &default_size_types),
          normalize_sizes(options.max_async_size, &default_size_types),
          normalize_sizes(options.max_size, &default_size_types),
        ]),
        max_initial_size: merge_sizes(vec![
          normalize_sizes(fallback_cache_group.max_initial_size, &default_size_types),
          normalize_sizes(fallback_cache_group.max_size, &default_size_types),
          normalize_sizes(options.max_initial_size, &default_size_types),
          normalize_sizes(options.max_size, &default_size_types),
        ]),
        automatic_name_delimiter: fallback_cache_group
          .automatic_name_delimiter
          .unwrap_or_else(|| {
            options
              .automatic_name_delimiter
              .clone()
              .unwrap_or_else(|| "~".to_string())
          }),
      },
    };

    let cache_group_source_by_key = {
      options
        .cache_groups
        .clone()
        .into_iter()
        .map(|(name, group_option)| {
          (
            name.clone(),
            create_cache_group_source(group_option, name, &default_size_types),
          )
        })
    }
    .collect::<HashMap<_, _>>();

    let cache_group_by_key = {
      cache_group_source_by_key.values().map(|group_source| {
        (
          group_source.key.clone(),
          create_cache_group(&normalized_options, group_source),
        )
      })
    }
    .collect::<HashMap<_, _>>();

    tracing::debug!(
      "created cache groups: {:#?}",
      cache_group_by_key.keys().collect::<Vec<_>>()
    );

    Self {
      _cache_group_source_by_key: cache_group_source_by_key,
      raw_options: options,
      cache_group_by_key,
    }
  }

  fn get_cache_groups(&self, module: &dyn Module) -> Vec<String> {
    self
      .raw_options
      .cache_groups
      .iter()
      .filter(|(_, group_option)| {
        // Align with
        group_option
          .test
          .as_ref()
          .map_or(true, |test| (test)(module))
          && group_option
            .r#type
            .as_ref()
            .map_or(true, |ty| ty == module.module_type())
        // TODO: check module layer
      })
      // TODO: Supports filter with module type
      .map(|(key, _group_option)| key.clone())
      .collect()
  }

  #[allow(clippy::format_in_format_args)]
  #[allow(clippy::too_many_arguments)]
  fn add_module_to_chunks_info_map(
    &self,
    cache_group: &CacheGroup,
    cache_group_index: usize,
    selected_chunks: &[&Chunk],
    // selectedChunksKey,
    module: &dyn Module,
    chunks_info_map: &mut HashMap<String, ChunksInfoItem>,
    named_chunk: &HashMap<String, ChunkUkey>,
    chunk_by_ukey: &rspack_core::ChunkByUkey,
    chunk_group_by_ukey: &ChunkGroupByUkey,
    // compilation: &mut Compilation,
  ) {
    let module_identifier = module.identifier();
    // Break if minimum number of chunks is not reached
    if selected_chunks.len() < cache_group.min_chunks as usize {
      tracing::debug!(
        "[Bailout-Module]: {}, because selected_chunks.len({:?}) < cache_group.min_chunks({:?})",
        module_identifier,
        selected_chunks.len(),
        cache_group.min_chunks
      );
      return;
    }

    // Determine name for split chunk
    let name = (cache_group.get_name)(module);

    let existing_chunk = name.clone().and_then(|name| {
      named_chunk
        .get(&name)
        .and_then(|chunk_ukey| chunk_by_ukey.get(chunk_ukey))
    });
    if let Some(existing_chunk) = existing_chunk {
      // Module can only be moved into the existing chunk if the existing chunk
      // is a parent of all selected chunks
      let mut is_in_all_parents = true;
      let queue = selected_chunks
        .iter()
        .flat_map(|c| {
          let groups = c
            .groups
            .iter()
            .filter_map(|ukey| chunk_group_by_ukey.get(ukey))
            .collect::<Vec<_>>();
          let ancestors = groups
            .iter()
            .flat_map(|g| g.ancestors(chunk_group_by_ukey))
            .collect::<Vec<_>>();
          groups.into_iter().map(|g| g.ukey).chain(ancestors)
        })
        .collect::<HashSet<_>>();

      for group in queue {
        let group = chunk_group_by_ukey.get(&group).expect("group should exist");
        if existing_chunk.is_in_group(&group.ukey) {
          continue;
        }
        is_in_all_parents = false;
        break;
      }
      let valid = is_in_all_parents;
      if !valid {
        panic!("{}{}{}{}{}",
            "SplitChunksPlugin\n",
            format!("Cache group \"{}\" conflicts with existing chunk.\n", cache_group.key),
            format!("Both have the same name \"{name:?}\" and existing chunk is not a parent of the selected modules.\n"),
            "Use a different name for the cache group or make sure that the existing chunk is a parent (e. g. via dependOn).\n",
            "HINT: You can omit \"name\" to automatically create a name.\n",
        )
      }
    }

    let key = format!(
      "{} {}",
      cache_group.key.clone(),
      if let Some(name) = &name {
        format!("name:{name}")
      } else {
        format!("chunk:{}", {
          let mut keys = selected_chunks
            .iter()
            .map(|c| c.ukey.as_usize().to_string())
            .collect::<Vec<_>>();
          keys.sort_unstable();
          keys.join("_")
        })
      }
    );

    let info = chunks_info_map
      .entry(key)
      .or_insert_with(|| ChunksInfoItem {
        modules: Default::default(),
        cache_group: cache_group.key.clone(),
        cache_group_index,
        name,
        sizes: Default::default(),
        chunks: Default::default(),
        _reuseable_chunks: Default::default(),
      });
    let old_size = info.modules.len();
    info.modules.insert(module.identifier());

    if info.modules.len() != old_size {
      module.source_types().iter().for_each(|ty| {
        let sizes = info.sizes.entry(*ty).or_default();
        *sizes += module.size(ty);
      });
    }

    info.chunks.extend(
      selected_chunks
        .iter()
        .map(|chunk| chunk.ukey)
        .collect::<HashSet<_>>(),
    );
  }

  fn create_chunks_info_map(
    &self,
    compilation: &mut Compilation,
  ) -> HashMap<String, ChunksInfoItem> {
    let mut chunks_info_map: HashMap<String, ChunksInfoItem> = Default::default();

    for module in compilation.module_graph.modules().values() {
      let cache_group_source_keys = self.get_cache_groups(module.as_ref());
      if cache_group_source_keys.is_empty() {
        tracing::debug!(
          "[Bailout-No matched groups]: Module({})",
          module.identifier(),
        );
        continue;
      }
      tracing::debug!(
        "Module({}) witch matched groups {:?}",
        module.identifier(),
        cache_group_source_keys
      );

      let mut cache_group_index = 0;
      for cache_group_source in cache_group_source_keys {
        let cache_group = self
          .cache_group_by_key
          .get(&cache_group_source)
          .expect("TODO:");
        let combs = vec![compilation
          .chunk_graph
          .get_module_chunks(module.identifier())];

        for combinations in combs {
          if combinations.len() < cache_group.min_chunks as usize {
            tracing::debug!(
              "[Bailout]: CacheGroup({}), because of combinations({:?}) < cache_group.min_chunks({:?})",
              cache_group.key,
              combinations.len(),
              cache_group.min_chunks
            );
            continue;
          }

          let selected_chunks = combinations
            .iter()
            .filter_map(|c| compilation.chunk_by_ukey.get(c))
            .filter(|c| (cache_group.chunks_filter)(c, &compilation.chunk_group_by_ukey))
            .collect::<Vec<_>>();

          tracing::debug!(
            "Split Module({}) with selected_chunks {:?} into group '{}'",
            module.identifier(),
            selected_chunks.iter().map(|c| c.ukey).collect::<Vec<_>>(),
            cache_group.key,
          );
          self.add_module_to_chunks_info_map(
            cache_group,
            cache_group_index,
            &selected_chunks,
            module.as_ref(),
            &mut chunks_info_map,
            &compilation.named_chunks,
            &compilation.chunk_by_ukey, // compilation,
            &compilation.chunk_group_by_ukey,
          );
        }

        cache_group_index += 1;
      }
    }
    chunks_info_map
  }

  fn find_best_entry(&self, chunks_info_map: &mut ChunksInfoMap) -> (String, ChunksInfoItem) {
    let mut chunks_info_map_iter = chunks_info_map.iter();
    let (best_entry_key, mut best_entry) = chunks_info_map_iter.next().expect("item should exist");
    let mut best_entry_key = best_entry_key.clone();
    for (key, info) in chunks_info_map_iter {
      if compare_entries(best_entry, info, &self.cache_group_by_key) < 0f64 {
        best_entry_key = key.clone();
        best_entry = info;
      }
    }

    (
      best_entry_key.clone(),
      chunks_info_map
        .remove(&best_entry_key)
        .expect("item should exist"),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  #[allow(clippy::if_same_then_else)]
  fn find_reusable_chunk(
    &self,
    compilation: &Compilation,
    item: &ChunksInfoItem,
    mut new_chunk: Option<ChunkUkey>,
  ) -> Option<ChunkUkey> {
    'outer: for chunk in &item.chunks {
      if compilation.chunk_graph.get_number_of_chunk_modules(chunk) != item.modules.len() {
        continue;
      }

      if item.chunks.len() > 1 && compilation.chunk_graph.get_number_of_entry_modules(chunk) > 0 {
        continue;
      }

      for module in &item.modules {
        if !compilation.chunk_graph.is_module_in_chunk(module, *chunk) {
          continue 'outer;
        }
      }

      let chunk = compilation
        .chunk_by_ukey
        .get(chunk)
        .expect("chunk should exist");
      if new_chunk.is_none()
        || new_chunk
          .and_then(|ukey| compilation.chunk_by_ukey.get(&ukey))
          .as_ref()
          .map_or(false, |c| c.name.is_none())
      {
        new_chunk = Some(chunk.ukey);
      } else if chunk.name.as_ref().map_or(false, |chunk_name| {
        new_chunk
          .and_then(|new_ukey| compilation.chunk_by_ukey.get(&new_ukey))
          .and_then(|c| c.name.as_ref())
          .map_or(false, |new_chunk_name| {
            chunk_name.len() < new_chunk_name.len()
          })
      }) {
        new_chunk = Some(chunk.ukey);
      } else if chunk.name.as_ref().map_or(false, |chunk_name| {
        new_chunk
          .and_then(|new_ukey| compilation.chunk_by_ukey.get(&new_ukey))
          .and_then(|c| c.name.as_ref())
          .map_or(false, |new_chunk_name| {
            chunk_name.len() == new_chunk_name.len() && chunk_name < new_chunk_name
          })
      }) {
        new_chunk = Some(chunk.ukey);
      };
    }
    new_chunk
  }

  fn remove_unrelated_chunk(
    &self,
    compilation: &Compilation,
    used_chunks: &mut HashSet<ChunkUkey>,
    item: &ChunksInfoItem,
  ) {
    'outer: for chunk in &used_chunks.clone() {
      for module in &item.modules {
        if compilation.chunk_graph.is_module_in_chunk(module, *chunk) {
          continue 'outer;
        } else {
          used_chunks.remove(chunk);
        }
      }
    }
  }

  fn filter_items_lesser_than_min_size(
    &self,
    chunks_info_map: &mut ChunksInfoMap,
    compilation: &mut Compilation,
  ) {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/optimize/SplitChunksPlugin.js#L1280
    let mut to_be_removed: HashSet<String> = HashSet::default();
    for (key, info) in chunks_info_map.iter_mut() {
      if remove_min_size_violating_modules(
        info,
        &self.cache_group_by_key,
        &mut compilation.module_graph,
      ) || !check_min_size_reduction(
        &info.sizes,
        &self
          .cache_group_by_key
          .get(&info.cache_group)
          .expect("TODO:")
          .min_size_reduction,
        info.chunks.len(),
      ) {
        to_be_removed.insert(key.clone());
      }
    }
    to_be_removed.into_iter().for_each(|cache_group_key| {
      let info = chunks_info_map.remove(&cache_group_key);
      tracing::debug!(
        "Remove cache group '{:?}' because of minSize violation",
        info
      );
    });
  }

  fn remove_all_modules_from_other_entries_and_update_size(
    &self,
    item: &mut ChunksInfoItem,
    chunks_info_map: &mut ChunksInfoMap,
    used_chunks: &mut HashSet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    let mut to_be_deleted = HashSet::default();
    // remove all modules from other entries and update size
    for (key, info) in chunks_info_map.iter_mut() {
      let is_overlap = info.chunks.union(used_chunks).next().is_some();
      if is_overlap {
        // update modules and total size
        // may remove it from the map when < minSize
        let mut updated = false;
        for module in &item.modules {
          if info.modules.contains(module) {
            info.modules.remove(module);
            let module = compilation
              .module_graph
              .module_by_identifier(module)
              .unwrap_or_else(|| panic!("Module({module}) not found"));
            for ty in module.source_types() {
              let sizes = info.sizes.get_mut(ty).unwrap_or_else(|| {
                panic!(
                  "{:?} is not existed in sizes of {} for module({})",
                  ty,
                  info.cache_group,
                  module.identifier()
                )
              });
              *sizes -= module.size(ty);
            }
          }
          updated = true;
        }

        if updated {
          if info.modules.is_empty() {
            to_be_deleted.insert(key.to_string());
            continue;
          }
          if remove_min_size_violating_modules(
            info,
            &self.cache_group_by_key,
            &mut compilation.module_graph,
          ) || !check_min_size_reduction(
            &info.sizes,
            &info
              .cache_group(&self.cache_group_by_key)
              .min_size_reduction,
            info.chunks.len(),
          ) {
            to_be_deleted.insert(key.to_string());
            continue;
          }
        }
      }
    }

    to_be_deleted.into_iter().for_each(|key| {
      chunks_info_map.remove(&key);
    });
  }

  fn link_module_new_chunk_and_remove_in_old_chunks(
    &self,
    is_reused_with_all_modules: bool,
    item: &ChunksInfoItem,
    new_chunk: ChunkUkey,
    used_chunks: &HashSet<ChunkUkey>,
    compilation: &mut Compilation,
  ) {
    if !is_reused_with_all_modules {
      // Add all modules to the new chunk
      for module_identifier in &item.modules {
        // TODO: module.chunkCondition
        // Add module to new chunk
        compilation
          .chunk_graph
          .connect_chunk_and_module(new_chunk, *module_identifier);
        // Remove module from used chunks
        for used_chunk in used_chunks {
          let used_chunk = compilation
            .chunk_by_ukey
            .get(used_chunk)
            .expect("Chunk should exist");
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&used_chunk.ukey, *module_identifier);
        }
      }
    } else {
      // Remove all modules from used chunks
      for module_identifier in &item.modules {
        for used_chunk in used_chunks {
          let used_chunk = compilation
            .chunk_by_ukey
            .get(used_chunk)
            .expect("Chunk should exist");
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&used_chunk.ukey, *module_identifier);
        }
      }
    }
  }

  fn split_used_chunks(
    &self,
    used_chunks: &HashSet<ChunkUkey>,
    new_chunk: ChunkUkey,
    compilation: &mut Compilation,
  ) {
    let new_chunk_ukey = new_chunk;
    for used_chunk in used_chunks {
      let [new_chunk, used_chunk] = compilation
        .chunk_by_ukey
        ._todo_should_remove_this_method_inner_mut()
        .get_many_mut([&new_chunk_ukey, used_chunk])
        .expect("TODO:");
      used_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);
    }
  }
}

#[async_trait::async_trait]
impl Plugin for SplitChunksPlugin {
  fn name(&self) -> &'static str {
    "split_chunks"
  }

  #[allow(clippy::unwrap_in_result)]
  #[allow(clippy::if_same_then_else)]
  #[allow(clippy::collapsible_else_if)]
  #[allow(unused)]
  async fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    let compilation = args.compilation;

    let mut chunks_info_map: HashMap<String, ChunksInfoItem> =
      self.create_chunks_info_map(compilation);

    // Filter items were size < minSize
    self.filter_items_lesser_than_min_size(&mut chunks_info_map, compilation);

    let split_chunks_span = tracing::trace_span!("split chunks with chunks_info_map");
    split_chunks_span.in_scope(|| {
      while !chunks_info_map.is_empty() {
        let (best_entry_key, mut item) = self.find_best_entry(&mut chunks_info_map);
        let item_cache_group = item.cache_group(&self.cache_group_by_key);

        let mut chunk_name = item.name.clone();
        let mut new_chunk: Option<ChunkUkey> = None;
        let mut is_existing_chunk = false;
        let mut is_reused_with_all_modules = false;
        if let Some(chunk_name) = chunk_name.clone() {
          let chunk_by_name = compilation.named_chunks.get(&chunk_name);
          if let Some(chunk_by_name) = chunk_by_name {
            let chunk = compilation
              .chunk_by_ukey
              .get_mut(chunk_by_name)
              .expect("chunk should exist");
            let old_size = item.chunks.len();
            item.chunks.remove(&chunk.ukey);
            is_existing_chunk = item.chunks.len() != old_size;
            new_chunk = Some(chunk.ukey);
          }
        } else if item_cache_group.reuse_existing_chunk {
          new_chunk = self.find_reusable_chunk(compilation, &item, new_chunk);
          if let Some(new_chunk) = new_chunk {
            item.chunks.remove(&new_chunk);
            chunk_name = None;
            is_existing_chunk = true;
            is_reused_with_all_modules = true;
          }
        };

        let enforced = check_min_size(&item.sizes, &item_cache_group.min_size);

        let mut used_chunks = item.chunks.clone();

        // Check if maxRequests condition can be fulfilled
        if !enforced
          && (item
            .cache_group(&self.cache_group_by_key)
            .max_initial_requests
            == u32::MAX
            || item
              .cache_group(&self.cache_group_by_key)
              .max_async_requests
              == u32::MAX)
        {
          for chunk in used_chunks.clone() {
            let chunk = compilation
              .chunk_by_ukey
              .get(&chunk)
              .expect("Chunk not found");
            let max_requests = if chunk.is_only_initial(&compilation.chunk_group_by_ukey) {
              item_cache_group.max_initial_requests
            } else {
              if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
                u32::min(
                  item
                    .cache_group(&self.cache_group_by_key)
                    .max_initial_requests,
                  item
                    .cache_group(&self.cache_group_by_key)
                    .max_async_requests,
                )
              } else {
                item
                  .cache_group(&self.cache_group_by_key)
                  .max_async_requests
              }
            };
            if u32::MAX == max_requests
              && get_requests(chunk, &compilation.chunk_group_by_ukey) > max_requests
            {
              used_chunks.remove(&chunk.ukey);
            }
          }
        }

        self.remove_unrelated_chunk(compilation, &mut used_chunks, &item);

        // Were some (invalid) chunks removed from usedChunks?
        // => readd all modules to the queue, as things could have been changed
        if used_chunks.len() < item.chunks.len() {
          if is_existing_chunk {
            used_chunks.insert(*new_chunk.as_ref().expect("New chunk not found"));
          }
          if used_chunks.len() >= item_cache_group.min_chunks as usize {
            let chunk_arr = used_chunks
              .iter()
              .filter_map(|ukey| compilation.chunk_by_ukey.get(ukey))
              .collect::<Vec<_>>();
            for module in &item.modules {
              self.add_module_to_chunks_info_map(
                item_cache_group,
                item.cache_group_index,
                &chunk_arr,
                &**compilation
                  .module_graph
                  .module_by_identifier(module)
                  .expect("Module not found"),
                &mut chunks_info_map,
                &compilation.named_chunks,
                &compilation.chunk_by_ukey, // compilation,
                &compilation.chunk_group_by_ukey,
              )
            }
          }
          continue;
        };

        // Validate minRemainingSize constraint when a single chunk is left over
        if !enforced && item_cache_group.validate_remaining_size && used_chunks.len() == 1 {
          let chunk = used_chunks.iter().next().expect("Chunk should exist");
          let mut chunk_sizes = SplitChunkSizes::default();
          for module in compilation
            .chunk_graph
            .get_chunk_modules(chunk, &compilation.module_graph)
          {
            let module = compilation
              .module_graph
              .module_by_identifier(&module.identifier())
              .expect("Module should exist");
            if !item.modules.contains(&module.identifier()) {
              for ty in module.source_types() {
                let sizes = chunk_sizes.entry(*ty).or_default();
                *sizes += module.size(ty);
              }
            }
          }
          let violating_sizes = get_violating_min_sizes(
            &chunk_sizes,
            &item
              .cache_group(&self.cache_group_by_key)
              .min_remaining_size,
          );
          if let Some(violating_sizes) = violating_sizes {
            let old_modules_size = item.modules.len();
            remove_modules_with_source_type(
              &mut item,
              &violating_sizes,
              &mut compilation.module_graph,
            );
            if !item.modules.is_empty() && item.modules.len() != old_modules_size {
              // queue this item again to be processed again
              // without violating modules
              chunks_info_map.insert(best_entry_key, item);
            }
            continue;
          }
        }

        // Create the new chunk if not reusing one
        let new_chunk = if let Some(existed) = new_chunk {
          existed
        } else {
          if let Some(chunk_name) = &chunk_name {
            Compilation::add_named_chunk(
              chunk_name.clone(),
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            )
            .ukey
          } else {
            tracing::debug!(
              "create a non-named chunk for cache group {}",
              item_cache_group.key
            );
            Compilation::add_chunk(&mut compilation.chunk_by_ukey).ukey
          }
        };

        compilation.chunk_graph.add_chunk(new_chunk);

        // Walk through all chunks
        let new_chunk_ukey = new_chunk;
        self.split_used_chunks(&used_chunks, new_chunk, compilation);

        let new_chunk = compilation
          .chunk_by_ukey
          .get_mut(&new_chunk_ukey)
          .expect("Chunk should exist");
        let new_chunk_ukey = new_chunk.ukey;
        new_chunk.chunk_reasons.push(if is_reused_with_all_modules {
          "reused as split chunk".to_string()
        } else {
          "split chunk".to_string()
        });

        new_chunk
          .chunk_reasons
          .push(format!("(cache group: {})", item_cache_group.key));

        if let Some(chunk_name) = &chunk_name {
          new_chunk
            .chunk_reasons
            .push(format!("(name: {chunk_name})"));
        }

        // new_chunk.id_name_hints.insert(info)

        self.link_module_new_chunk_and_remove_in_old_chunks(
          is_reused_with_all_modules,
          &item,
          new_chunk.ukey,
          &used_chunks,
          compilation,
        );

        let mut max_size_queue_map: HashMap<ChunkUkey, MaxSizeQueueItem> = Default::default();

        if !item
          .cache_group(&self.cache_group_by_key)
          .max_async_size
          .is_empty()
          || !item
            .cache_group(&self.cache_group_by_key)
            .max_initial_size
            .is_empty()
        {
          let old_max_size_settings = max_size_queue_map.remove(&new_chunk_ukey);
          max_size_queue_map.insert(
            new_chunk_ukey,
            MaxSizeQueueItem {
              min_size: old_max_size_settings
                .as_ref()
                .map(|old| {
                  combine_sizes(
                    &old.min_size,
                    &item
                      .cache_group(&self.cache_group_by_key)
                      .min_size_for_max_size,
                    f64::max,
                  )
                })
                .unwrap_or_else(|| item_cache_group.min_size.clone()),
              max_async_size: old_max_size_settings
                .as_ref()
                .map(|old| {
                  combine_sizes(
                    &old.max_async_size,
                    &item_cache_group.max_async_size,
                    f64::min,
                  )
                })
                .unwrap_or_else(|| {
                  item
                    .cache_group(&self.cache_group_by_key)
                    .max_async_size
                    .clone()
                }),
              max_initial_size: old_max_size_settings
                .as_ref()
                .map(|old| {
                  combine_sizes(
                    &old.max_initial_size,
                    &item_cache_group.max_initial_size,
                    f64::min,
                  )
                })
                .unwrap_or_else(|| {
                  item
                    .cache_group(&self.cache_group_by_key)
                    .max_initial_size
                    .clone()
                }),
              keys: old_max_size_settings
                .map(|mut old| {
                  old.keys.push(item_cache_group.key.clone());
                  old.keys
                })
                .unwrap_or_else(|| vec![item_cache_group.key.clone()]),
            },
          );
        }

        // remove all modules from other entries and update size
        self.remove_all_modules_from_other_entries_and_update_size(
          &mut item,
          &mut chunks_info_map,
          &mut used_chunks,
          compilation,
        )
      }
    });

    // Make sure that maxSize is fulfilled
    // let fallbackCacheGroup = self.options.f

    Ok(())
  }
}
