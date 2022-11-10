#![allow(unused_variables)]

use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use rspack_core::{Chunk, ChunkUkey, Compilation, ModuleGraphModule, Plugin};

use crate::{
  CacheGroup, CacheGroupOptions, CacheGroupSource, SizeType, SplitChunkSizes, SplitChunksOptions,
};

pub fn create_cache_group(group_source: &CacheGroupSource) -> CacheGroup {
  let min_size = {
    let mut cloned = group_source.min_size.clone();
    if !group_source.enforce {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let min_size_reduction = {
    let mut cloned = group_source.min_size_reduction.clone();
    if !group_source.enforce {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let min_remaining_size = {
    let mut cloned = group_source.min_remaining_size.clone();
    if !group_source.enforce {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let enforce_size_threshold = {
    let mut cloned = group_source.enforce_size_threshold.clone();
    if !group_source.enforce {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let max_async_size = {
    let mut cloned = group_source.max_async_size.clone();
    if !group_source.enforce {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let max_initial_size = {
    let mut cloned = group_source.max_initial_size.clone();
    if !group_source.enforce {
      cloned.extend(group_source.enforce_size_threshold.clone());
    }
    cloned
  };
  let get_name = group_source.get_name.clone();
  let chunks_filter = group_source.chunks_filter.clone();
  CacheGroup {
    key: group_source.key.clone(),
    priority: group_source.priority,
    chunks_filter,
    min_size,
    min_size_reduction,
    min_remaining_size,
    enforce_size_threshold,
    max_async_size,
    max_initial_size,
    min_chunks: group_source.min_chunks,
    max_async_requests: group_source.max_async_requests,
    max_initial_requests: group_source.max_initial_requests,
    get_name,
    // used_exports: group_source.used_exports,
    automatic_name_delimiter: group_source.automatic_name_delimiter.clone(),
    filename: group_source.filename.clone(),
    id_hint: group_source.id_hint.clone(),
    reuse_existing_chunk: group_source.reuse_existing_chunk,
  }
}

#[derive(Debug)]
pub struct SplitChunksPlugin {
  options: SplitChunksOptions,
  _cache_group_source_by_key: HashMap<String, CacheGroupSource>,
  cache_group_by_key: HashMap<String, CacheGroup>,
}

pub fn create_cache_group_source(
  options: CacheGroupOptions,
  key: String,
  default_size_types: &[SizeType],
) -> CacheGroupSource {
  let min_size = {
    default_size_types
      .iter()
      .map(|size_type| (*size_type, options.min_size))
      .collect::<HashMap<_, _>>()
  };

  let min_size_reduction = {
    default_size_types
      .iter()
      .map(|size_type| (*size_type, options.min_size_reduction))
      .collect::<HashMap<_, _>>()
  };

  let max_size = {
    default_size_types
      .iter()
      .map(|size_type| (*size_type, options.max_size))
      .collect::<HashMap<_, _>>()
  };

  let enforce_size_threshold = {
    default_size_types
      .iter()
      .map(|size_type| (*size_type, options.enforce_size_threshold))
      .collect::<HashMap<_, _>>()
  };
  let max_async_size = {
    default_size_types
      .iter()
      .map(|size_type| (*size_type, options.max_async_size))
      .collect::<HashMap<_, _>>()
  };

  let max_initial_size = {
    default_size_types
      .iter()
      .map(|size_type| (*size_type, options.max_initial_size))
      .collect::<HashMap<_, _>>()
  };

  let min_remaining_size = {
    let mut tmp = default_size_types
      .iter()
      .map(|size_type| (*size_type, options.min_remaining_size))
      .collect::<HashMap<_, _>>();
    tmp.extend(min_size.clone());
    tmp
  };
  let get_name = {
    let name = options.name.clone();
    Arc::new(move || name.clone())
  };

  CacheGroupSource {
    key,
    priority: options.priority,
    get_name,
    chunks_filter: Arc::new(move |_chunk| match options.chunks {
      crate::ChunkType::Initial => todo!("Supports initial"),
      crate::ChunkType::Async => todo!("Supports async"),
      crate::ChunkType::All => true,
      // crate::ChunkType::Custom(_) => todo!(),
    }),
    enforce: options.enforce,
    min_size,
    min_size_reduction,
    min_remaining_size,
    enforce_size_threshold,
    max_async_size,
    max_initial_size,
    min_chunks: options.min_chunks,
    max_async_requests: options.max_async_requests,
    max_initial_requests: options.max_initial_requests,
    filename: options.filename.clone(),
    id_hint: options.id_hint.clone(),
    automatic_name_delimiter: options.automatic_name_delimiter.clone(),
    reuse_existing_chunk: options.reuse_existing_chunk,
    // used_exports: options.used_exports,
  }
}

impl SplitChunksPlugin {
  pub fn new(options: SplitChunksOptions) -> Self {
    let cache_group_source_by_key = {
      options
        .cache_groups
        .clone()
        .into_iter()
        .map(|(name, group_option)| {
          (
            name.clone(),
            create_cache_group_source(group_option, name, &options.default_size_types),
          )
        })
    }
    .collect::<HashMap<_, _>>();

    let cache_group_by_key = {
      cache_group_source_by_key
        .values()
        .map(|group_source| (group_source.key.clone(), create_cache_group(group_source)))
    }
    .collect::<HashMap<_, _>>();

    Self {
      options,
      _cache_group_source_by_key: cache_group_source_by_key,
      cache_group_by_key,
    }
  }

  fn get_cache_groups(&self, module: &ModuleGraphModule) -> Vec<String> {
    self
      .options
      .cache_groups
      .values()
      .filter(|group_option| (group_option.test)(module))
      // TODO: Supports filter with module type
      .map(|group_option| group_option.name.clone())
      .collect()
  }

  fn add_module_to_chunks_info_map(
    &self,
    cache_group: &CacheGroup,
    cache_group_index: usize,
    selected_chunks: &[&Chunk],
    // selectedChunksKey,
    module_identifier: String,
    chunks_info_map: &mut HashMap<String, ChunksInfoItem>,
    // compilation: &mut Compilation,
  ) {
    if selected_chunks.len() < cache_group.min_chunks {
      return;
    }

    let name = (cache_group.get_name)();
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
          &mut chunks_info_map,
          // compilation,
        );
        cache_group_index += 1;
      }
    }

    for (key, info) in chunks_info_map.into_iter() {
      let chunk_name = info.name.clone();
      let new_chunk = Compilation::add_chunk(
        &mut compilation.chunk_by_ukey,
        Some(chunk_name.clone()),
        chunk_name.clone(),
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
