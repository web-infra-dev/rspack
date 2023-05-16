#![allow(clippy::obfuscated_if_else)]
#![allow(clippy::comparison_chain)]

use std::sync::Arc;

use rspack_core::{Module, ModuleGraph, SourceType};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  chunks_info_item::ChunksInfoItem,
  utils::{get_violating_min_sizes, merge_sizes2, normalize_sizes},
  CacheGroup, CacheGroupOptions, CacheGroupSource, ChunkFilterFn, ChunkType, NormalizedOptions,
  SizeType, SplitChunksNameFn,
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
  let chunks_filter = group_source.chunks_filter.clone();
  CacheGroup {
    key: group_source.key.clone(),
    priority: group_source.priority.unwrap_or(0),
    chunks_filter: chunks_filter
      .clone()
      .unwrap_or_else(|| options.chunk_filter.clone()),
    min_size: min_size.clone(),
    min_size_reduction,
    min_remaining_size: min_remaining_size.clone(),
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
    min_size_for_max_size: merge_sizes2(group_source.min_size.clone(), options.min_size.clone()),
    validate_remaining_size: min_remaining_size.values().any(|size| size > &0f64),
  }
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
    .map(|name| Arc::new(move |_: &dyn Module| Some(name.clone())) as SplitChunksNameFn);

  CacheGroupSource {
    key,
    priority: options.priority,
    get_name,
    chunks_filter: options.chunks.map(|chunks| {
      let f: ChunkFilterFn = Arc::new(move |chunk, chunk_group_by_ukey| match chunks {
        ChunkType::Initial => chunk.can_be_initial(chunk_group_by_ukey),
        ChunkType::Async => !chunk.can_be_initial(chunk_group_by_ukey),
        ChunkType::All => true,
      });
      f
    }),
    enforce: options.enforce,
    min_size: min_size.clone(),
    min_size_reduction,
    min_remaining_size: merge_sizes2(
      normalize_sizes(options.min_remaining_size, default_size_types),
      min_size,
    ),
    enforce_size_threshold: normalize_sizes(options.enforce_size_threshold, default_size_types),
    max_async_size: merge_sizes2(
      normalize_sizes(options.max_async_size, default_size_types),
      max_size.clone(),
    ),
    max_initial_size: merge_sizes2(
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

pub(crate) fn remove_modules_with_source_type(
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

pub(crate) fn remove_min_size_violating_modules(
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
