#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::obfuscated_if_else)]
#![allow(clippy::comparison_chain)]

use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  sync::Arc,
};

use rspack_core::{
  Chunk, ChunkGroupByUkey, ChunkUkey, Compilation, Module, ModuleGraph, ModuleIdentifier, Plugin,
  SourceType,
};

use crate::{
  CacheGroup, CacheGroupOptions, CacheGroupSource, ChunkFilter, ChunkType, GetName,
  NormalizedOptions, SizeType, SplitChunkSizes, SplitChunksOptions,
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

  let get_name = options.name.map(|name| {
    Arc::new(move |m: &dyn Module| name.clone()) as Arc<dyn Fn(&dyn Module) -> String + Send + Sync>
  });

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
      let get_name: GetName = Arc::new(move |module: &dyn Module| name.clone().expect("TODO:"));
      get_name
    };
    let normalized_options = NormalizedOptions {
      default_size_types: default_size_types.clone(),
      min_size: min_size.clone(),
      min_size_reduction,
      min_remaining_size: merge_sizes(
        normalize_sizes(options.min_remaining_size, &default_size_types),
        min_size,
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

  fn add_module_to_chunks_info_map(
    &self,
    cache_group: &CacheGroup,
    cache_group_index: usize,
    selected_chunks: &[&Chunk],
    // selectedChunksKey,
    module_identifier: ModuleIdentifier,
    module_graph_module: &dyn Module,
    chunks_info_map: &mut HashMap<String, ChunksInfoItem>,
    // compilation: &mut Compilation,
  ) {
    if selected_chunks.len() < cache_group.min_chunks {
      tracing::debug!(
        "[Bailout-Module]: {}, because selected_chunks.len({:?}) < cache_group.min_chunks({:?})",
        module_identifier,
        selected_chunks.len(),
        cache_group.min_chunks
      );
      return;
    }

    let name = (cache_group.get_name)(module_graph_module);
    // let existing_chunk = compilation
    //   .named_chunk
    //   .get(&name)
    //   .and_then(|chunk_ukey| compilation.chunk_by_ukey.get(chunk_ukey));
    // if existing_chunk.is_some() {
    //   panic!("TODO: Supports reuse existing chunk");
    // }

    let key = cache_group.key.clone();

    let info = chunks_info_map
      .entry(key)
      .or_insert_with(|| ChunksInfoItem {
        modules: Default::default(),
        cache_group: cache_group.key.clone(),
        _cache_group_index: cache_group_index,
        name,
        sizes: Default::default(),
        chunks: Default::default(),
        _reuseable_chunks: Default::default(),
      });

    info.modules.insert(module_identifier);
    info.chunks.extend(
      selected_chunks
        .iter()
        .map(|chunk| chunk.ukey)
        .collect::<HashSet<_>>(),
    );
  }
}

#[derive(Debug)]
struct ChunksInfoItem {
  // Sortable Module Set
  pub modules: HashSet<ModuleIdentifier>,
  pub cache_group: String,
  pub _cache_group_index: usize,
  pub name: String,
  pub sizes: SplitChunkSizes,
  pub chunks: HashSet<ChunkUkey>,
  pub _reuseable_chunks: HashSet<ChunkUkey>,
  // bigint | Chunk
  // pub chunks_keys: Hash
}

impl Plugin for SplitChunksPlugin {
  fn name(&self) -> &'static str {
    "split_chunks"
  }

  #[allow(clippy::unwrap_in_result)]
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
          "[Bailout-Module]: '{}', because {}",
          module.identifier(),
          "no cache group matched"
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
              "[Bailout-CacheGroup]: '{}', because of combinations({:?}) < cache_group.min_chunks({:?})",
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
            // compilation,
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

    // TODO: Filter items were size < minSize
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

    for (key, info) in chunks_info_map.into_iter() {
      let chunk_name = info.name.clone();
      let is_chunk_existing = compilation.named_chunks.get(&chunk_name).is_some();
      let new_chunk = Compilation::add_named_chunk(
        chunk_name.clone(),
        &mut compilation.chunk_by_ukey,
        &mut compilation.named_chunks,
      );
      if is_chunk_existing {
        tracing::debug!("Reuse a existing Chunk({})", chunk_name);
      } else {
        tracing::debug!("Create a new Chunk({})", chunk_name);
      }

      compilation.chunk_graph.add_chunk(new_chunk.ukey);

      let used_chunks = &info
        .chunks
        .iter()
        .filter(|chunk_ukey| {
          // Chunks containing at least one related module are used.
          info.modules.iter().any(|module_identifier| {
            compilation
              .chunk_graph
              .is_module_in_chunk(module_identifier, **chunk_ukey)
          })
        })
        .collect::<Vec<_>>();

      if used_chunks.len() != info.chunks.len() {
        tracing::debug!(
          "Drop {:?} unused chunks",
          info.chunks.len() - used_chunks.len(),
        );
      }
      let new_chunk_ukey = new_chunk.ukey;
      for used_chunk in used_chunks {
        let [new_chunk, used_chunk] = compilation
          .chunk_by_ukey
          .get_many_mut([&new_chunk_ukey, used_chunk])
          .expect("TODO:");
        used_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);

        for module_identifier in &info.modules {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&used_chunk.ukey, module_identifier);
        }
      }
      for module_identifier in info.modules {
        compilation
          .chunk_graph
          .connect_chunk_and_module(new_chunk_ukey, module_identifier);
      }
    }

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
