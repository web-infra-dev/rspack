#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::obfuscated_if_else)]
#![allow(clippy::comparison_chain)]

use std::{collections::HashSet, fmt::Debug, sync::Arc};

use hashbrown::HashMap;
use rspack_core::{
  Chunk, ChunkGroupByUkey, ChunkUkey, Compilation, Module, ModuleGraph, ModuleIdentifier, Plugin,
  SourceType,
};

use crate::{
  utils::{check_min_size, compare_entries},
  CacheGroup, CacheGroupByKey, CacheGroupOptions, CacheGroupSource, ChunkFilter, ChunkType,
  GetName, NormalizedFallbackCacheGroup, NormalizedOptions, SizeType, SplitChunkSizes,
  SplitChunksOptions,
};

pub fn create_cache_group(
  options: &NormalizedOptions,
  group_source: &CacheGroupSource,
) -> CacheGroup {
  let min_size = {
    let mut cloned = group_source.min_size.clone();
    if !group_source.enforce.unwrap_or_default() {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let min_size_reduction = {
    let mut cloned = group_source.min_size_reduction.clone();
    if !group_source.enforce.unwrap_or_default() {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let min_remaining_size = {
    let mut cloned = group_source.min_remaining_size.clone();
    if !group_source.enforce.unwrap_or_default() {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let enforce_size_threshold = {
    let mut cloned = group_source.enforce_size_threshold.clone();
    if !group_source.enforce.unwrap_or_default() {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let max_async_size = {
    let mut cloned = group_source.max_async_size.clone();
    if !group_source.enforce.unwrap_or_default() {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let max_initial_size = {
    let mut cloned = group_source.max_initial_size.clone();
    if !group_source.enforce.unwrap_or_default() {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let get_name = group_source.get_name.clone();
  let chunks_filter = group_source.chunks_filter.clone();
  CacheGroup {
    key: group_source.key.clone(),
    priority: group_source.priority.unwrap_or(0),
    chunks_filter: chunks_filter
      .clone()
      .unwrap_or_else(|| options.chunk_filter.clone()),
    min_size: min_size.clone(),
    min_size_reduction,
    min_remaining_size,
    enforce_size_threshold,
    max_async_size,
    max_initial_size,
    min_chunks: group_source.min_chunks.unwrap_or_else(|| {
      group_source
        .enforce
        .unwrap_or_default()
        .then_some(1)
        .unwrap_or(options.min_chunks)
    }),
    max_async_requests: group_source.max_async_requests.unwrap_or_else(|| {
      group_source
        .enforce
        .unwrap_or_default()
        .then_some(1)
        .unwrap_or(options.max_async_requests)
    }),
    max_initial_requests: group_source.max_initial_requests.unwrap_or_else(|| {
      group_source
        .enforce
        .unwrap_or_default()
        .then_some(1)
        .unwrap_or(options.max_initial_requests)
    }),
    get_name: group_source
      .get_name
      .clone()
      .unwrap_or_else(|| options.get_name.clone()),
    // used_exports: group_source.used_exports,
    automatic_name_delimiter: group_source.automatic_name_delimiter.clone(),
    filename: group_source
      .filename
      .clone()
      .map(Some)
      .unwrap_or_else(|| options.filename.clone()),
    id_hint: group_source
      .id_hint
      .clone()
      .unwrap_or_else(|| group_source.key.clone()),
    reuse_existing_chunk: group_source.reuse_existing_chunk.unwrap_or_default(),
    validate_size: min_size.values().any(|size| size > &0f64),
    min_size_for_max_size: merge_sizes(group_source.min_size.clone(), options.min_size.clone()),
  }
}

#[derive(Debug)]
pub struct SplitChunksPlugin {
  raw_options: SplitChunksOptions,
  options: NormalizedOptions,
  _cache_group_source_by_key: HashMap<String, CacheGroupSource>,
  cache_group_by_key: HashMap<String, CacheGroup>,
}

pub fn create_cache_group_source(
  options: CacheGroupOptions,
  key: String,
  default_size_types: &[SizeType],
) -> CacheGroupSource {
  let min_size = normalize_sizes(options.min_size, default_size_types);
  let min_size_reduction = normalize_sizes(options.min_size_reduction, default_size_types);
  let max_size = normalize_sizes(options.max_size, default_size_types);

  let get_name = options
    .name
    .map(|name| Arc::new(move |m: &dyn Module| Some(name.clone())) as GetName);

  CacheGroupSource {
    key,
    priority: options.priority,
    get_name,
    chunks_filter: options.chunks.map(|chunks| {
      let f: ChunkFilter = Arc::new(move |chunk, chunk_group_by_ukey| match chunks {
        ChunkType::Initial => chunk.can_be_initial(chunk_group_by_ukey),
        ChunkType::Async => !chunk.can_be_initial(chunk_group_by_ukey),
        ChunkType::All => true,
      });
      f
    }),
    enforce: options.enforce,
    min_size: min_size.clone(),
    min_size_reduction,
    min_remaining_size: merge_sizes(
      normalize_sizes(options.min_remaining_size, default_size_types),
      min_size,
    ),
    enforce_size_threshold: normalize_sizes(options.enforce_size_threshold, default_size_types),
    max_async_size: merge_sizes(
      normalize_sizes(options.max_async_size, default_size_types),
      max_size.clone(),
    ),
    max_initial_size: merge_sizes(
      normalize_sizes(options.max_initial_size, default_size_types),
      max_size,
    ),
    min_chunks: options.min_chunks,
    max_async_requests: options.max_async_requests,
    max_initial_requests: options.max_initial_requests,
    filename: options.filename.clone(),
    id_hint: options.id_hint.clone(),
    automatic_name_delimiter: options
      .automatic_name_delimiter
      .clone()
      .unwrap_or_else(|| "~".to_string()),
    reuse_existing_chunk: options.reuse_existing_chunk,
    // used_exports: options.used_exports,
  }
}

fn normalize_sizes<T: Clone>(
  value: Option<T>,
  default_size_types: &[SizeType],
) -> HashMap<SizeType, T> {
  value
    .map(|value| {
      default_size_types
        .iter()
        .cloned()
        .map(|size_type| (size_type, value.clone()))
        .collect::<HashMap<_, _>>()
    })
    .unwrap_or_default()
}

fn merge_sizes(mut a: HashMap<SizeType, f64>, b: HashMap<SizeType, f64>) -> HashMap<SizeType, f64> {
  a.extend(b);
  a
}

impl SplitChunksPlugin {
  pub fn new(options: SplitChunksOptions) -> Self {
    let default_size_types = options
      .default_size_types
      .clone()
      .unwrap_or_else(|| vec![SizeType::JavaScript, SizeType::Unknown]);

    let min_size = normalize_sizes(options.min_size, &default_size_types);
    let min_size_reduction = normalize_sizes(options.min_size_reduction, &default_size_types);
    let max_size = normalize_sizes(options.max_size, &default_size_types);

    let enforce_size_threshold =
      normalize_sizes(options.enforce_size_threshold, &default_size_types);
    let max_async_size = normalize_sizes(options.max_async_size, &default_size_types);
    let max_initial_size = normalize_sizes(options.max_initial_size, &default_size_types);
    let min_remaining_size = normalize_sizes(options.min_remaining_size, &default_size_types);

    let get_name = {
      let name = options.name.clone();
      let get_name: GetName = Arc::new(move |module: &dyn Module| name.clone());
      get_name
    };
    let normalized_options = NormalizedOptions {
      default_size_types: default_size_types.clone(),
      min_size: min_size.clone(),
      min_size_reduction,
      min_remaining_size: merge_sizes(
        normalize_sizes(options.min_remaining_size, &default_size_types),
        min_size.clone(),
      ),
      enforce_size_threshold: normalize_sizes(options.enforce_size_threshold, &default_size_types),
      max_async_size: merge_sizes(
        normalize_sizes(options.max_async_size, &default_size_types),
        max_size.clone(),
      ),
      max_initial_size: merge_sizes(
        normalize_sizes(options.max_initial_size, &default_size_types),
        max_size,
      ),
      min_chunks: options.min_chunks.unwrap_or(1),
      max_async_requests: options.min_chunks.unwrap_or(1),
      max_initial_requests: options.min_chunks.unwrap_or(1),
      filename: None,
      get_name,
      chunk_filter: {
        let chunks = options.chunks;
        Arc::new(move |chunk, chunk_group_by_ukey| {
          let chunk_type = chunks.as_ref().unwrap_or(&ChunkType::Async);
          chunk_type.is_selected(chunk, chunk_group_by_ukey)
        })
      },
      fallback_cache_group: NormalizedFallbackCacheGroup {
        chunks_filter: {
          let chunks = options
            .fallback_cache_group
            .as_ref()
            .map(|f| f.chunks)
            .unwrap_or_else(|| options.chunks);
          Arc::new(move |chunk, chunk_group_by_ukey| {
            let chunk_type = chunks.as_ref().unwrap_or(&ChunkType::All);
            chunk_type.is_selected(chunk, chunk_group_by_ukey)
          })
        },
        min_size: merge_sizes(
          normalize_sizes(
            options
              .fallback_cache_group
              .as_ref()
              .and_then(|f| f.min_size),
            &default_size_types,
          ),
          min_size.clone(),
        ),
        max_async_size: merge_sizes(
          normalize_sizes(
            options
              .fallback_cache_group
              .as_ref()
              .map(|f| f.min_size)
              .unwrap_or_default(),
            &default_size_types,
          ),
          min_size.clone(),
        ),
        max_initial_size: merge_sizes(
          normalize_sizes(
            options
              .fallback_cache_group
              .as_ref()
              .map(|f| f.min_size)
              .unwrap_or_default(),
            &default_size_types,
          ),
          min_size,
        ),
        automatic_name_delimiter: options
          .fallback_cache_group
          .as_ref()
          .map(|f| f.automatic_name_delimiter.clone())
          .unwrap_or_else(|| options.automatic_name_delimiter.clone())
          .unwrap_or_else(|| "~".to_string()),
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
      options: normalized_options,
      _cache_group_source_by_key: cache_group_source_by_key,
      raw_options: options,
      cache_group_by_key,
    }
  }

  fn chunks_filter(&self, chunk: &Chunk, chunk_group_by_ukey: &ChunkGroupByUkey) -> bool {
    self
      .raw_options
      .chunks
      .unwrap_or(ChunkType::All)
      .is_selected(chunk, chunk_group_by_ukey)
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
    module_identifier: ModuleIdentifier,
    module_graph_module: &dyn Module,
    chunks_info_map: &mut HashMap<String, ChunksInfoItem>,
    named_chunk: &HashMap<String, ChunkUkey>,
    chunk_by_ukey: &HashMap<ChunkUkey, Chunk>,
    chunk_group_by_ukey: &ChunkGroupByUkey,
    // compilation: &mut Compilation,
  ) {
    // Break if minimum number of chunks is not reached
    if selected_chunks.len() < cache_group.min_chunks {
      tracing::debug!(
        "[Bailout-Module]: {}, because selected_chunks.len({:?}) < cache_group.min_chunks({:?})",
        module_identifier,
        selected_chunks.len(),
        cache_group.min_chunks
      );
      return;
    }

    // Determine name for split chunk
    let name = (cache_group.get_name)(module_graph_module);

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
            format!("Both have the same name \"{:?}\" and existing chunk is not a parent of the selected modules.\n", name),
            "Use a different name for the cache group or make sure that the existing chunk is a parent (e. g. via dependOn).\n",
            "HINT: You can omit \"name\" to automatically create a name.\n",
        )
      }
    }

    let key = cache_group.key.clone();

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
    info.modules.insert(module_identifier);

    if info.modules.len() != old_size {
      module_graph_module.source_types().iter().for_each(|ty| {
        let sizes = info.sizes.entry(*ty).or_default();
        *sizes += module_graph_module.size(ty);
      });
    }

    info.chunks.extend(
      selected_chunks
        .iter()
        .map(|chunk| chunk.ukey)
        .collect::<HashSet<_>>(),
    );
  }
}

#[derive(Debug)]
pub(crate) struct ChunksInfoItem {
  // Sortable Module Set
  pub modules: HashSet<ModuleIdentifier>,
  pub cache_group: String,
  pub cache_group_index: usize,
  pub name: Option<String>,
  pub sizes: SplitChunkSizes,
  pub chunks: HashSet<ChunkUkey>,
  pub _reuseable_chunks: HashSet<ChunkUkey>,
  // bigint | Chunk
  // pub chunks_keys: Hash
}

impl ChunksInfoItem {
  pub(crate) fn cache_group<'cache_group>(
    &self,
    map: &'cache_group CacheGroupByKey,
  ) -> &'cache_group CacheGroup {
    map
      .get(&self.cache_group)
      .unwrap_or_else(|| panic!("Cache group not found: {}", self.cache_group))
  }
}

impl Plugin for SplitChunksPlugin {
  fn name(&self) -> &'static str {
    "split_chunks"
  }

  #[allow(clippy::unwrap_in_result)]
  #[allow(clippy::if_same_then_else)]
  #[allow(clippy::collapsible_else_if)]
  #[allow(unused)]
  fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    let compilation = args.compilation;
    // let modules = compilation
    //   .module_graph
    //   .modules()
    //   .map(|m| m.uri.clone())
    //   .collect::<Vec<_>>();
    let mut chunks_info_map: HashMap<String, ChunksInfoItem> = Default::default();

    for module in compilation.module_graph.modules() {
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
          .get_modules_chunks(&module.identifier())];

        for combinations in combs {
          if combinations.len() < cache_group.min_chunks {
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
            module.identifier(),
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

    fn remove_modules_with_source_type(
      info: &mut ChunksInfoItem,
      source_types: &[SourceType],
      module_graph: &mut ModuleGraph,
    ) {
      info.modules.retain(|module_identifier| {
        let module = module_graph
          .module_by_identifier(module_identifier)
          .expect("module should exist");
        let types = module.source_types();
        if source_types.iter().any(|ty| types.contains(ty)) {
          info
            .sizes
            .iter_mut()
            .for_each(|(ty, size)| *size -= module.size(ty));
          true
        } else {
          false
        }
      });
    }

    fn remove_min_size_violating_modules(
      info: &mut ChunksInfoItem,
      cache_group_by_key: &HashMap<String, CacheGroup>,
      module_graph: &mut ModuleGraph,
    ) -> bool {
      let cache_group = cache_group_by_key
        .get(&info.cache_group)
        .expect("cache_group should exists");
      if !cache_group.validate_size {
        return false;
      };
      let violating_sizes = get_violating_min_sizes(&info.sizes, &cache_group.min_size);
      if let Some(violating_sizes) = violating_sizes {
        remove_modules_with_source_type(info, &violating_sizes, module_graph);
        info.modules.is_empty()
      } else {
        false
      }
    }

    // Filter items were size < minSize
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

    while !chunks_info_map.is_empty() {
      let mut chunks_info_map_iter = chunks_info_map.iter();
      let (best_entry_key, mut best_entry) =
        chunks_info_map_iter.next().expect("item should exist");
      let mut best_entry_key = best_entry_key.clone();
      for (key, info) in chunks_info_map_iter {
        if compare_entries(best_entry, info, &self.cache_group_by_key) < 0f64 {
          best_entry_key = key.clone();
          best_entry = info;
        }
      }

      let mut item = chunks_info_map
        .remove(&best_entry_key)
        .expect("item should exist");

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
      } else if item
        .cache_group(&self.cache_group_by_key)
        .reuse_existing_chunk
      {
        'outer: for chunk in &item.chunks {
          if compilation.chunk_graph.get_number_of_chunk_modules(chunk) != item.modules.len() {
            continue;
          }

          if item.chunks.len() > 1 && compilation.chunk_graph.get_number_of_entry_modules(chunk) > 0
          {
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
        if let Some(new_chunk) = new_chunk {
          item.chunks.remove(&new_chunk);
          chunk_name = None;
          is_existing_chunk = true;
          is_reused_with_all_modules = true;
        }
      };

      let enforced = check_min_size(
        &item.sizes,
        &item.cache_group(&self.cache_group_by_key).min_size,
      );

      let mut used_chunks = item.chunks.clone();

      // Check if maxRequests condition can be fulfilled
      if !enforced
        && (item
          .cache_group(&self.cache_group_by_key)
          .max_initial_requests
          .eq(&usize::MAX)
          || item
            .cache_group(&self.cache_group_by_key)
            .max_async_requests
            .eq(&usize::MAX))
      {
        for chunk in used_chunks.clone() {
          let chunk = compilation
            .chunk_by_ukey
            .get(&chunk)
            .expect("Chunk not found");
          let max_requests = if chunk.is_only_initial(&compilation.chunk_group_by_ukey) {
            item
              .cache_group(&self.cache_group_by_key)
              .max_initial_requests
          } else {
            if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
              usize::min(
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
          if usize::MAX == max_requests
            && get_requests(chunk, &compilation.chunk_group_by_ukey) > max_requests
          {
            used_chunks.remove(&chunk.ukey);
          }
        }
      }

      'outer: for chunk in &used_chunks {
        for module in &item.modules {
          if compilation.chunk_graph.is_module_in_chunk(module, *chunk) {
            continue 'outer;
          }
        }
      }

      // Were some (invalid) chunks removed from usedChunks?
      // => readd all modules to the queue, as things could have been changed
      if used_chunks.len() < item.chunks.len() {
        if is_existing_chunk {
          used_chunks.insert(*new_chunk.as_ref().expect("New chunk not found"));
        }
        if used_chunks.len() >= item.cache_group(&self.cache_group_by_key).min_chunks {
          let chunk_arr = used_chunks
            .iter()
            .filter_map(|ukey| compilation.chunk_by_ukey.get(ukey))
            .collect::<Vec<_>>();
          for module in &item.modules {
            self.add_module_to_chunks_info_map(
              item.cache_group(&self.cache_group_by_key),
              item.cache_group_index,
              &chunk_arr,
              *module,
              compilation
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

      // TODO: Validate minRemainingSize constraint when a single chunk is left over

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
          Compilation::add_chunk(&mut compilation.chunk_by_ukey).ukey
        }
      };

      compilation.chunk_graph.add_chunk(new_chunk);

      // Walk through all chunks
      let new_chunk_ukey = new_chunk;
      for used_chunk in &used_chunks {
        let [new_chunk, used_chunk] = compilation
          .chunk_by_ukey
          .get_many_mut([&new_chunk_ukey, used_chunk])
          .expect("TODO:");
        used_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);

        for module_identifier in &item.modules {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&used_chunk.ukey, module_identifier);
        }
      }

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
        .push(format!("(cache group: {})", item.cache_group));

      if let Some(chunk_name) = &chunk_name {
        new_chunk
          .chunk_reasons
          .push(format!("(name: {})", chunk_name));
      }

      // new_chunk.id_name_hints.insert(info)

      if !is_reused_with_all_modules {
        // Add all modules to the new chunk
        for module_identifier in &item.modules {
          // TODO: module.chunkCondition
          // Add module to new chunk
          compilation
            .chunk_graph
            .connect_chunk_and_module(new_chunk_ukey, *module_identifier);
          // Remove module from used chunks
          for used_chunk in &used_chunks {
            let used_chunk = compilation
              .chunk_by_ukey
              .get(used_chunk)
              .expect("Chunk should exist");
            for module_identifier in &item.modules {
              compilation
                .chunk_graph
                .disconnect_chunk_and_module(&used_chunk.ukey, module_identifier);
            }
          }
        }
      } else {
        // Remove all modules from used chunks
        for module_identifier in &item.modules {
          for used_chunk in &used_chunks {
            let used_chunk = compilation
              .chunk_by_ukey
              .get(used_chunk)
              .expect("Chunk should exist");
            for module_identifier in &item.modules {
              compilation
                .chunk_graph
                .disconnect_chunk_and_module(&used_chunk.ukey, module_identifier);
            }
          }
        }
      }

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
              .unwrap_or_else(|| item.cache_group(&self.cache_group_by_key).min_size.clone()),
            max_async_size: old_max_size_settings
              .as_ref()
              .map(|old| {
                combine_sizes(
                  &old.max_async_size,
                  &item.cache_group(&self.cache_group_by_key).max_async_size,
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
                  &item.cache_group(&self.cache_group_by_key).max_initial_size,
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
                old
                  .keys
                  .push(item.cache_group(&self.cache_group_by_key).key.clone());
                old.keys
              })
              .unwrap_or_else(|| vec![item.cache_group(&self.cache_group_by_key).key.clone()]),
          },
        );
      }

      let mut to_be_deleted = HashSet::new();
      // remove all modules from other entries and update size
      for (key, info) in &mut chunks_info_map {
        let is_overlap = info.chunks.union(&used_chunks).next().is_some();
        if is_overlap {
          // update modules and total size
          // may remove it from the map when < minSize
          let mut updated = false;
          for module in &item.modules {
            if info.modules.contains(module) {
              info.modules.remove(module);
            }
            let module = compilation
              .module_graph
              .module_by_identifier(module)
              .expect("module should exist");
            for key in module.source_types() {
              let sizes = info.sizes.get_mut(key).expect("size should exist");
              *sizes -= module.size(key);
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

    // Make sure that maxSize is fulfilled
    // let fallbackCacheGroup = self.options.f

    Ok(())
  }
}

fn get_violating_min_sizes(
  sizes: &SplitChunkSizes,
  min_size: &SplitChunkSizes,
) -> Option<Vec<SourceType>> {
  let mut list: Option<Vec<SourceType>> = None;
  for key in min_size.keys() {
    let size = sizes.get(key).unwrap_or(&0f64);
    if size == &0f64 {
      continue;
    };
    let min_size = min_size.get(key).unwrap_or(&0f64);
    if size < min_size {
      list.get_or_insert_default().push(*key);
    }
  }
  list
}

fn check_min_size_reduction(
  sizes: &SplitChunkSizes,
  min_size_reduction: &SplitChunkSizes,
  chunk_count: usize,
) -> bool {
  for key in min_size_reduction.keys() {
    let size = sizes.get(key).unwrap_or(&0f64);
    if size == &0f64 {
      continue;
    };
    let min_size_reduction = min_size_reduction.get(key).unwrap_or(&0f64);
    if (size * chunk_count as f64) < *min_size_reduction {
      return false;
    }
  }
  true
}

fn get_requests(chunk: &Chunk, chunk_group_by_ukey: &ChunkGroupByUkey) -> usize {
  let mut requests = 0;
  for group in &chunk.groups {
    let group = chunk_group_by_ukey
      .get(group)
      .expect("ChunkGroup not found");
    requests = usize::max(requests, group.chunks.len())
  }
  requests
}

#[derive(Debug)]
struct MaxSizeQueueItem {
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub keys: Vec<String>,
}

fn combine_sizes(
  a: &SplitChunkSizes,
  b: &SplitChunkSizes,
  combine: impl Fn(f64, f64) -> f64,
) -> SplitChunkSizes {
  let a_keys = a.keys();
  let b_keys = b.keys();
  let mut res: SplitChunkSizes = Default::default();
  for key in a_keys {
    if b.contains_key(key) {
      res.insert(*key, combine(a[key], b[key]));
    } else {
      res.insert(*key, a[key]);
    }
  }

  for key in b_keys {
    if !a.contains_key(key) {
      res.insert(*key, b[key]);
    }
  }

  res
}
