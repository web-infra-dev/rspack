use std::{collections::HashMap, sync::Arc};

use napi_derive::napi;
use rspack_core::SourceType;
use rspack_plugin_split_chunks::{CacheGroupOptions, ChunkType, SplitChunksOptions, TestFn};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSplitChunksOptions {
  pub fallback_cache_group: Option<RawFallbackCacheGroupOptions>,
  pub name: Option<String>,
  pub cache_groups: Option<HashMap<String, RawCacheGroupOptions>>,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  pub max_async_requests: Option<u32>,
  pub max_initial_requests: Option<u32>,
  //   pub default_size_types: Option<Vec<SizeType>>,
  pub min_chunks: Option<u32>,
  // hide_path_info: bool,
  pub min_size: Option<f64>,
  //   pub min_size_reduction: usize,
  pub enforce_size_threshold: Option<f64>,
  pub min_remaining_size: Option<f64>,
  // layer: String,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
}

impl From<RawSplitChunksOptions> for SplitChunksOptions {
  fn from(value: RawSplitChunksOptions) -> Self {
    let mut defaults = SplitChunksOptions {
      max_async_requests: value.max_async_requests,
      max_initial_requests: value.max_initial_requests,
      min_chunks: value.min_chunks,
      min_size: value.min_size,
      enforce_size_threshold: value.enforce_size_threshold,
      min_remaining_size: value.min_remaining_size,
      chunks: value.chunks.map(|chunks| match chunks.as_str() {
        "initial" => ChunkType::Initial,
        "async" => ChunkType::Async,
        "all" => ChunkType::All,
        _ => panic!("Invalid chunk type: {chunks}"),
      }),
      ..Default::default()
    };

    defaults
      .cache_groups
      .extend(
        value
          .cache_groups
          .unwrap_or_default()
          .into_iter()
          .map(|(k, v)| {
            (
              k,
              CacheGroupOptions {
                name: v.name,
                priority: v.priority,
                reuse_existing_chunk: Some(false),
                test: v.test.clone().map(|test| {
                  let f: TestFn = Arc::new(move |module| {
                    let re = rspack_regex::RspackRegex::new(&test)
                      .unwrap_or_else(|_| panic!("Invalid regex: {}", &test));
                    module
                      .name_for_condition()
                      .map_or(false, |name| re.test(&name))
                  });
                  f
                }),
                chunks: v.chunks.map(|chunks| match chunks.as_str() {
                  "initial" => ChunkType::Initial,
                  "async" => ChunkType::Async,
                  "all" => ChunkType::All,
                  _ => panic!("Invalid chunk type: {chunks}"),
                }),
                min_chunks: v.min_chunks,
                ..Default::default()
              },
            )
          }),
      );
    defaults
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCacheGroupOptions {
  pub priority: Option<i32>,
  // pub reuse_existing_chunk: Option<bool>,
  //   pub r#type: SizeType,
  pub test: Option<String>,
  //   pub filename: String,
  //   pub enforce: bool,
  //   pub id_hint: String,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  pub min_chunks: Option<u32>,
  // hide_path_info: bool,
  pub min_size: Option<f64>,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
  pub name: Option<String>,
  // used_exports: bool,
  pub reuse_existing_chunk: Option<bool>,
  pub enforce: Option<bool>,
}

use rspack_plugin_split_chunks_new as new_split_chunks_plugin;

impl From<RawSplitChunksOptions> for new_split_chunks_plugin::PluginOptions {
  fn from(raw_opts: RawSplitChunksOptions) -> Self {
    use new_split_chunks_plugin::{create_chunk_filter_from_str, SplitChunkSizes};

    let mut cache_groups = vec![];

    let overall_chunk_filter = raw_opts.chunks.as_deref().map(create_chunk_filter_from_str);

    let overall_min_chunks = raw_opts.min_chunks.unwrap_or(1);

    let overall_name_getter = raw_opts
      .name
      .map(new_split_chunks_plugin::create_chunk_name_getter_by_const_name)
      .unwrap_or_else(new_split_chunks_plugin::create_empty_chunk_name_getter);

    let default_size_types = [SourceType::JavaScript, SourceType::Unknown];

    let create_sizes = |size: Option<f64>| {
      size
        .map(|size| SplitChunkSizes::with_initial_value(&default_size_types, size))
        .unwrap_or_else(SplitChunkSizes::default)
    };

    let empty_sizes = SplitChunkSizes::empty();

    let overall_min_size = create_sizes(raw_opts.min_size);
    let overall_max_size = create_sizes(raw_opts.max_size);
    let overall_max_async_size = create_sizes(raw_opts.max_async_size).merge(&overall_max_size);
    let overall_max_initial_size = create_sizes(raw_opts.max_initial_size).merge(&overall_max_size);

    cache_groups.extend(
      raw_opts
        .cache_groups
        .unwrap_or_default()
        .into_iter()
        .map(|(key, v)| {
          let enforce = v.enforce.unwrap_or_default();

          let min_size = create_sizes(v.min_size).merge(if enforce {
            &empty_sizes
          } else {
            &overall_min_size
          });

          let max_size = create_sizes(v.max_size);

          let max_async_size = create_sizes(v.max_async_size)
            .merge(&max_size)
            .merge(if enforce {
              &empty_sizes
            } else {
              &overall_max_async_size
            });

          let max_initial_size =
            create_sizes(v.max_initial_size)
              .merge(&max_size)
              .merge(if enforce {
                &empty_sizes
              } else {
                &overall_max_initial_size
              });

          let min_chunks = v
            .min_chunks
            .unwrap_or(if enforce { 1 } else { overall_min_chunks });

          new_split_chunks_plugin::CacheGroup {
            id_hint: key.clone(),
            key,
            name: v
              .name
              .map(new_split_chunks_plugin::create_chunk_name_getter_by_const_name)
              .unwrap_or_else(|| overall_name_getter.clone()),
            priority: v.priority.unwrap_or(-20) as f64,
            test: new_split_chunks_plugin::create_module_filter(v.test.clone()),
            chunk_filter: v
              .chunks
              .as_deref()
              .map(rspack_plugin_split_chunks_new::create_chunk_filter_from_str)
              .unwrap_or_else(|| {
                overall_chunk_filter
                  .clone()
                  .unwrap_or_else(rspack_plugin_split_chunks_new::create_async_chunk_filter)
              }),
            min_chunks,
            min_size,
            reuse_existing_chunk: v.reuse_existing_chunk.unwrap_or(true),
            // TODO(hyf0): the non-enforced default value should be 30
            // I would set align default value with Webpack when the options is exposed to users
            max_async_requests: u32::MAX,
            max_initial_requests: u32::MAX,
            max_async_size,
            max_initial_size,
          }
        }),
    );

    let raw_fallback_cache_group = raw_opts.fallback_cache_group.unwrap_or_default();

    let fallback_chunks_filter = raw_fallback_cache_group
      .chunks
      .as_deref()
      .map(create_chunk_filter_from_str);

    let fallback_min_size =
      create_sizes(raw_fallback_cache_group.min_size).merge(&overall_min_size);

    let fallback_max_size = create_sizes(raw_fallback_cache_group.max_size);

    let fallback_max_async_size = create_sizes(raw_fallback_cache_group.max_async_size)
      .merge(&fallback_max_size)
      .merge(&overall_max_async_size)
      .merge(&overall_max_size);

    let fallback_max_initial_size = create_sizes(raw_fallback_cache_group.max_initial_size)
      .merge(&fallback_max_size)
      .merge(&overall_max_initial_size)
      .merge(&overall_max_size);

    new_split_chunks_plugin::PluginOptions {
      cache_groups,
      fallback_cache_group: rspack_plugin_split_chunks_new::FallbackCacheGroup {
        chunks_filter: fallback_chunks_filter.unwrap_or_else(|| {
          overall_chunk_filter
            .clone()
            .unwrap_or_else(rspack_plugin_split_chunks_new::create_all_chunk_filter)
        }),
        min_size: fallback_min_size,
        max_async_size: fallback_max_async_size,
        max_initial_size: fallback_max_initial_size,
      },
    }
  }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawFallbackCacheGroupOptions {
  pub chunks: Option<String>,
  pub min_size: Option<f64>,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
}
