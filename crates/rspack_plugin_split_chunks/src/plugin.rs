#![allow(unused_variables)]

use std::{
  clone,
  collections::{HashMap, HashSet},
  fmt::Debug,
  sync::Arc,
};

use rspack_core::{Chunk, ChunkGroupByUkey, ChunkUkey, Compilation, ModuleGraphModule, Plugin};

use crate::{
  CacheGroup, CacheGroupOptions, CacheGroupSource, ChunkType, SizeType, SplitChunkSizes,
  SplitChunksOptions,
};

pub fn create_cache_group(
  options: &SplitChunksPluginOptions,
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
    chunks_filter,
    min_size,
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
      .unwrap_or(options.filename.clone().unwrap()),
    id_hint: group_source
      .id_hint
      .clone()
      .unwrap_or_else(|| group_source.key.clone()),
    reuse_existing_chunk: group_source.reuse_existing_chunk.unwrap_or_default(),
  }
}

#[derive(Debug)]
pub struct SplitChunksPlugin {
  raw_options: SplitChunksOptions,
  options: SplitChunksPluginOptions,
  _cache_group_source_by_key: HashMap<String, CacheGroupSource>,
  cache_group_by_key: HashMap<String, CacheGroup>,
}

pub struct SplitChunksPluginOptions {
  default_size_types: Vec<SizeType>,
  pub min_size: SplitChunkSizes,
  pub min_size_reduction: SplitChunkSizes,
  pub min_remaining_size: SplitChunkSizes,
  pub enforce_size_threshold: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub min_chunks: usize,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub filename: Option<String>,
  pub get_name: Arc<dyn Fn(&ModuleGraphModule) -> String + Send + Sync>,
}

impl Debug for SplitChunksPluginOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SplitChunksPluginOptions")
      .field("default_size_types", &self.default_size_types)
      .field("min_size", &self.min_size)
      .field("min_size_reduction", &self.min_size_reduction)
      .field("min_remaining_size", &self.min_remaining_size)
      .field("enforce_size_threshold", &self.enforce_size_threshold)
      .field("max_async_size", &self.max_async_size)
      .field("max_initial_size", &self.max_initial_size)
      .field("min_chunks", &self.min_chunks)
      .field("max_async_requests", &self.max_async_requests)
      .field("max_initial_requests", &self.max_initial_requests)
      .field("filename", &self.filename)
      .field("get_name", &"Fn")
      .finish()
  }
}

pub fn create_cache_group_source(
  options: CacheGroupOptions,
  key: String,
  default_size_types: &[SizeType],
) -> CacheGroupSource {
  let min_size = normalize_sizes(options.min_size, &default_size_types);
  let min_size_reduction = normalize_sizes(options.min_size_reduction, &default_size_types);
  let max_size = normalize_sizes(options.max_size, &default_size_types);

  let get_name = options.name.map(|name| {
    Arc::new(move |m: &ModuleGraphModule| name.clone())
      as Arc<dyn Fn(&ModuleGraphModule) -> String + Send + Sync>
  });

  CacheGroupSource {
    key,
    priority: options.priority.clone(),
    get_name: get_name,
    chunks_filter: Arc::new(move |_chunk| {
      options
        .chunks
        .clone()
        .map(|chunk_type| match chunk_type {
          crate::ChunkType::Initial => todo!("Supports initial"),
          crate::ChunkType::Async => todo!("Supports async"),
          crate::ChunkType::All => true,
          // crate::ChunkType::Custom(_) => todo!(),
        })
        .unwrap_or_default()
    }),
    enforce: options.enforce,
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
    min_chunks: options.min_chunks,
    max_async_requests: options.max_async_requests,
    max_initial_requests: options.max_initial_requests,
    filename: options.filename.clone(),
    id_hint: options.id_hint.clone(),
    automatic_name_delimiter: options.automatic_name_delimiter.clone().unwrap(),
    reuse_existing_chunk: options.reuse_existing_chunk,
    // used_exports: options.used_exports,
  }
}

fn normalize_sizes(
  value: Option<usize>,
  default_size_types: &[SizeType],
) -> HashMap<SizeType, usize> {
  value
    .map(|value| {
      default_size_types
        .iter()
        .map(|size_type| (*size_type, value))
        .collect::<HashMap<_, _>>()
    })
    .unwrap_or_default()
}

fn merge_sizes(
  mut a: HashMap<SizeType, usize>,
  b: HashMap<SizeType, usize>,
) -> HashMap<SizeType, usize> {
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
      Arc::new(move |module: &ModuleGraphModule| name.clone().unwrap())
        as Arc<dyn Fn(&ModuleGraphModule) -> String + Send + Sync>
    };
    let normalized_options = SplitChunksPluginOptions {
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
      get_name: get_name,
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
      .clone()
      .unwrap_or(ChunkType::All)
      .is_selected(chunk, chunk_group_by_ukey)
  }

  fn get_cache_groups(&self, module: &ModuleGraphModule) -> Vec<String> {
    self
      .raw_options
      .cache_groups
      .iter()
      .filter(|(_, group_option)| {
        group_option
          .test
          .as_ref()
          .map_or(true, |test| (test)(module))
        // TODO: we should also check type chunk type
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
    module_identifier: String,
    module_graph_module: &ModuleGraphModule,
    chunks_info_map: &mut HashMap<String, ChunksInfoItem>,
    // compilation: &mut Compilation,
  ) {
    if selected_chunks.len() < cache_group.min_chunks {
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
        _cache_group: cache_group.key.clone(),
        _cache_group_index: cache_group_index,
        name,
        _sizes: Default::default(),
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

struct ChunksInfoItem {
  // Sortable Module Set
  pub modules: HashSet<String>,
  pub _cache_group: String,
  pub _cache_group_index: usize,
  pub name: String,
  pub _sizes: SplitChunkSizes,
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

    for module in compilation.module_graph.module_graph_modules() {
      let cache_group_source_keys = self.get_cache_groups(module);
      if cache_group_source_keys.is_empty() {
        continue;
      }
      tracing::debug!("cache_group_source_keys {:?}", cache_group_source_keys);

      let mut cache_group_index = 0;
      for cache_group_source in cache_group_source_keys {
        let cache_group = self.cache_group_by_key.get(&cache_group_source).unwrap();
        let combinations = compilation
          .chunk_graph
          .get_modules_chunks(&module.module_identifier);
        if combinations.len() < cache_group.min_chunks {
          tracing::debug!(
            "bailout because of combinations.len() {:?} < {:?} cache_group.min_chunks",
            combinations.len(),
            cache_group.min_chunks
          );
          continue;
        }

        let selected_chunks = combinations
          .iter()
          .filter_map(|c| compilation.chunk_by_ukey.get(c))
          .filter(|c| (cache_group.chunks_filter)(c))
          .collect::<Vec<_>>();

        tracing::debug!("selected_chunks {:?}", selected_chunks);
        self.add_module_to_chunks_info_map(
          cache_group,
          cache_group_index,
          &selected_chunks,
          module.module_identifier.clone(),
          &module,
          &mut chunks_info_map,
          // compilation,
        );
        cache_group_index += 1;
      }
    }

    for (key, info) in chunks_info_map.into_iter() {
      let chunk_name = info.name.clone();
      let new_chunk = Compilation::add_named_chunk(
        chunk_name.clone(),
        chunk_name.clone(),
        &mut compilation.chunk_by_ukey,
        &mut compilation.named_chunks,
      );
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
      let new_chunk_ukey = new_chunk.ukey;
      for used_chunk in used_chunks {
        let [new_chunk, used_chunk] = compilation
          .chunk_by_ukey
          .get_many_mut([&new_chunk_ukey, used_chunk])
          .unwrap();
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
