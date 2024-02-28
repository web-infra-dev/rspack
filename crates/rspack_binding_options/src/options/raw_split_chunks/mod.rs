mod raw_split_chunk_cache_group_test;
mod raw_split_chunk_chunks;
mod raw_split_chunk_name;

use std::sync::Arc;

use derivative::Derivative;
use napi::{Either, JsString};
use napi_derive::napi;
use raw_split_chunk_name::normalize_raw_chunk_name;
use raw_split_chunk_name::RawChunkOptionName;
use rspack_core::Filename;
use rspack_core::SourceType;
use rspack_core::DEFAULT_DELIMITER;
use rspack_napi_shared::{JsRegExp, JsRegExpExt, JsStringExt};
use rspack_plugin_split_chunks::ChunkNameGetter;
use serde::Deserialize;

use self::raw_split_chunk_cache_group_test::default_cache_group_test;
use self::raw_split_chunk_cache_group_test::normalize_raw_cache_group_test;
use self::raw_split_chunk_cache_group_test::RawCacheGroupTest;
use self::raw_split_chunk_chunks::{create_chunks_filter, Chunks};
use self::raw_split_chunk_name::default_chunk_option_name;

#[derive(Derivative, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[derivative(Debug)]
pub struct RawSplitChunksOptions {
  pub fallback_cache_group: Option<RawFallbackCacheGroupOptions>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | false | Function")]
  #[derivative(Debug = "ignore")]
  pub name: Option<RawChunkOptionName>,
  pub cache_groups: Option<Vec<RawCacheGroupOptions>>,
  /// What kind of chunks should be selected.
  #[serde(skip_deserializing)]
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all' | Function")]
  #[derivative(Debug = "ignore")]
  pub chunks: Option<Chunks>,
  pub automatic_name_delimiter: Option<String>,
  pub max_async_requests: Option<u32>,
  pub max_initial_requests: Option<u32>,
  pub default_size_types: Vec<String>,
  pub min_chunks: Option<u32>,
  pub hide_path_info: Option<bool>,
  pub min_size: Option<f64>,
  //   pub min_size_reduction: usize,
  pub enforce_size_threshold: Option<f64>,
  pub min_remaining_size: Option<f64>,
  // layer: String,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
}

#[derive(Derivative, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[derivative(Debug)]
pub struct RawCacheGroupOptions {
  pub key: String,
  pub priority: Option<i32>,
  // pub reuse_existing_chunk: Option<bool>,
  //   pub r#type: SizeType,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "RegExp | string | Function")]
  #[derivative(Debug = "ignore")]
  pub test: Option<RawCacheGroupTest>,
  pub filename: Option<String>,
  //   pub enforce: bool,
  pub id_hint: Option<String>,
  /// What kind of chunks should be selected.
  #[serde(skip_deserializing)]
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all'")]
  #[derivative(Debug = "ignore")]
  pub chunks: Option<Chunks>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "RegExp | string")]
  #[derivative(Debug = "ignore")]
  pub r#type: Option<Either<JsRegExp, JsString>>,
  pub automatic_name_delimiter: Option<String>,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  pub min_chunks: Option<u32>,
  pub min_size: Option<f64>,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
  #[serde(skip_deserializing)]
  #[napi(ts_type = "string | false | Function")]
  #[derivative(Debug = "ignore")]
  pub name: Option<RawChunkOptionName>,
  // used_exports: bool,
  pub reuse_existing_chunk: Option<bool>,
  pub enforce: Option<bool>,
}

impl From<RawSplitChunksOptions> for rspack_plugin_split_chunks::PluginOptions {
  fn from(raw_opts: RawSplitChunksOptions) -> Self {
    use rspack_plugin_split_chunks::SplitChunkSizes;

    let mut cache_groups = vec![];

    let overall_chunk_filter = raw_opts.chunks.map(create_chunks_filter);

    let overall_min_chunks = raw_opts.min_chunks.unwrap_or(1);

    let overall_name_getter = raw_opts.name.map_or(default_chunk_option_name(), |name| {
      normalize_raw_chunk_name(name)
    });

    let default_size_types = raw_opts
      .default_size_types
      .into_iter()
      .map(|size_type| SourceType::from(size_type.as_str()))
      .collect::<Vec<_>>();

    let create_sizes = |size: Option<f64>| {
      size
        .map(|size| SplitChunkSizes::with_initial_value(&default_size_types, size))
        .unwrap_or_default()
    };

    let empty_sizes = SplitChunkSizes::empty();

    let overall_min_size = create_sizes(raw_opts.min_size);
    let overall_max_size = create_sizes(raw_opts.max_size);
    let overall_max_async_size = create_sizes(raw_opts.max_async_size).merge(&overall_max_size);
    let overall_max_initial_size = create_sizes(raw_opts.max_initial_size).merge(&overall_max_size);
    let overall_automatic_name_delimiter = raw_opts
      .automatic_name_delimiter
      .unwrap_or(DEFAULT_DELIMITER.to_string());

    cache_groups.extend(
      raw_opts
        .cache_groups
        .unwrap_or_default()
        .into_iter()
        .map(|v| {
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

          let min_chunks = if enforce {
            1
          } else {
            v.min_chunks.unwrap_or(overall_min_chunks)
          };

          let r#type = v
            .r#type
            .map(create_module_type_filter)
            .unwrap_or_else(rspack_plugin_split_chunks::create_default_module_type_filter);

          let mut name = v.name.map_or(default_chunk_option_name(), |name| {
            normalize_raw_chunk_name(name)
          });
          if matches!(name, ChunkNameGetter::Disabled) {
            name = overall_name_getter.clone();
          }
          rspack_plugin_split_chunks::CacheGroup {
            id_hint: v.id_hint.unwrap_or_else(|| v.key.clone()),
            key: v.key,
            name,
            priority: v.priority.unwrap_or(0) as f64,
            test: v.test.map_or(default_cache_group_test(), |test| {
              normalize_raw_cache_group_test(test)
            }),
            chunk_filter: v.chunks.map(create_chunks_filter).unwrap_or_else(|| {
              overall_chunk_filter
                .clone()
                .unwrap_or_else(rspack_plugin_split_chunks::create_async_chunk_filter)
            }),
            min_chunks,
            min_size,
            automatic_name_delimiter: v
              .automatic_name_delimiter
              .unwrap_or(overall_automatic_name_delimiter.clone()),
            filename: v.filename.map(Filename::from),
            reuse_existing_chunk: v.reuse_existing_chunk.unwrap_or(true),
            // TODO(hyf0): the non-enforced default value should be 30
            // I would set align default value with Webpack when the options is exposed to users
            max_async_requests: u32::MAX,
            max_initial_requests: u32::MAX,
            max_async_size,
            max_initial_size,
            r#type,
          }
        }),
    );

    let raw_fallback_cache_group = raw_opts.fallback_cache_group.unwrap_or_default();

    let fallback_chunks_filter = raw_fallback_cache_group.chunks.map(create_chunks_filter);

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

    rspack_plugin_split_chunks::PluginOptions {
      cache_groups,
      fallback_cache_group: rspack_plugin_split_chunks::FallbackCacheGroup {
        chunks_filter: fallback_chunks_filter.unwrap_or_else(|| {
          overall_chunk_filter
            .clone()
            .unwrap_or_else(rspack_plugin_split_chunks::create_all_chunk_filter)
        }),
        min_size: fallback_min_size,
        max_async_size: fallback_max_async_size,
        max_initial_size: fallback_max_initial_size,
        automatic_name_delimiter: raw_fallback_cache_group
          .automatic_name_delimiter
          .unwrap_or(overall_automatic_name_delimiter.clone()),
      },
      hide_path_info: raw_opts.hide_path_info,
    }
  }
}

#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[derivative(Debug)]
pub struct RawFallbackCacheGroupOptions {
  #[serde(skip_deserializing)]
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all'")]
  #[derivative(Debug = "ignore")]
  pub chunks: Option<Chunks>,
  pub min_size: Option<f64>,
  pub max_size: Option<f64>,
  pub max_async_size: Option<f64>,
  pub max_initial_size: Option<f64>,
  pub automatic_name_delimiter: Option<String>,
}

fn create_module_type_filter(
  raw: Either<JsRegExp, JsString>,
) -> rspack_plugin_split_chunks::ModuleTypeFilter {
  match raw {
    Either::A(js_reg) => {
      let regex = js_reg.to_rspack_regex();
      Arc::new(move |m| regex.test(m.module_type().as_str()))
    }
    Either::B(js_str) => {
      let type_str = js_str.into_string();
      Arc::new(move |m| m.module_type().as_str() == type_str.as_str())
    }
  }
}
