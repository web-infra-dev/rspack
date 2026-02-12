mod raw_split_chunk_cache_group_test;
mod raw_split_chunk_chunks;
mod raw_split_chunk_name;
mod raw_split_chunk_size;

use std::sync::Arc;

use derive_more::Debug;
use napi::{Either, JsString, bindgen_prelude::Either3};
use napi_derive::napi;
use raw_split_chunk_name::{RawChunkOptionName, normalize_raw_chunk_name};
use rspack_core::{DEFAULT_DELIMITER, Filename, SourceType};
use rspack_napi::{string::JsStringExt, threadsafe_function::ThreadsafeFunction};
use rspack_plugin_split_chunks::ChunkNameGetter;
use rspack_regex::RspackRegex;

use self::{
  raw_split_chunk_cache_group_test::{
    RawCacheGroupTest, default_cache_group_test, normalize_raw_cache_group_test,
  },
  raw_split_chunk_chunks::{Chunks, create_chunks_filter},
  raw_split_chunk_name::default_chunk_option_name,
  raw_split_chunk_size::RawSplitChunkSizes,
};
use crate::filename::JsFilename;

#[napi(object, object_to_js = false)]
#[derive(Debug)]
pub struct RawSplitChunksOptions<'a> {
  pub fallback_cache_group: Option<RawFallbackCacheGroupOptions<'a>>,
  #[napi(ts_type = "string | false | Function")]
  #[debug(skip)]
  pub name: Option<RawChunkOptionName>,
  pub filename: Option<JsFilename>,
  pub cache_groups: Option<Vec<RawCacheGroupOptions<'a>>>,
  /// What kind of chunks should be selected.
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all' | Function")]
  #[debug(skip)]
  pub chunks: Option<Chunks<'a>>,
  pub used_exports: Option<bool>,
  pub automatic_name_delimiter: Option<String>,
  pub max_async_requests: Option<f64>,
  pub max_initial_requests: Option<f64>,
  pub default_size_types: Vec<String>,
  pub min_chunks: Option<u32>,
  pub hide_path_info: Option<bool>,
  pub min_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub min_size_reduction: Option<Either<f64, RawSplitChunkSizes>>,
  //   pub min_size_reduction: usize,
  pub enforce_size_threshold: Option<f64>,
  pub min_remaining_size: Option<Either<f64, RawSplitChunkSizes>>,
  // layer: String,
  pub max_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_async_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_initial_size: Option<Either<f64, RawSplitChunkSizes>>,
}

#[napi(object, object_to_js = false)]
#[derive(Debug)]
pub struct RawCacheGroupOptions<'a> {
  pub key: String,
  pub priority: Option<i32>,
  // pub reuse_existing_chunk: Option<bool>,
  //   pub r#type: SizeType,
  #[napi(ts_type = "RegExp | string | Function")]
  #[debug(skip)]
  pub test: Option<RawCacheGroupTest>,
  pub filename: Option<JsFilename>,
  //   pub enforce: bool,
  pub id_hint: Option<String>,
  /// What kind of chunks should be selected.
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all'")]
  #[debug(skip)]
  pub chunks: Option<Chunks<'a>>,
  #[napi(ts_type = "RegExp | string")]
  #[debug(skip)]
  pub r#type: Option<Either<RspackRegex, JsString<'a>>>,
  #[napi(ts_type = "RegExp | string | ((layer?: string) => boolean)")]
  #[debug(skip)]
  pub layer: Option<Either3<RspackRegex, JsString<'a>, ThreadsafeFunction<Option<String>, bool>>>,
  pub automatic_name_delimiter: Option<String>,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  pub min_chunks: Option<u32>,
  pub min_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub min_size_reduction: Option<Either<f64, RawSplitChunkSizes>>,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  pub max_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_async_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_initial_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_async_requests: Option<f64>,
  pub max_initial_requests: Option<f64>,
  #[napi(ts_type = "string | false | Function")]
  #[debug(skip)]
  pub name: Option<RawChunkOptionName>,
  // used_exports: bool,
  pub reuse_existing_chunk: Option<bool>,
  pub enforce: Option<bool>,
  pub used_exports: Option<bool>,
}

impl<'a> From<RawSplitChunksOptions<'a>> for rspack_plugin_split_chunks::PluginOptions {
  fn from(raw_opts: RawSplitChunksOptions) -> Self {
    use rspack_plugin_split_chunks::SplitChunkSizes;

    let mut cache_groups = vec![];

    let overall_filename = raw_opts.filename.map(Filename::from);

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

    let create_sizes = |size: Option<Either<f64, RawSplitChunkSizes>>| match size {
      Some(Either::A(size)) => SplitChunkSizes::with_initial_value(&default_size_types, size),
      Some(Either::B(sizes)) => sizes.into(),
      None => SplitChunkSizes::default(),
    };

    let empty_sizes = SplitChunkSizes::empty();

    let overall_min_size = create_sizes(raw_opts.min_size);

    let overall_min_size_reduction = create_sizes(raw_opts.min_size_reduction);

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

          let min_size_reduction = create_sizes(v.min_size_reduction).merge(if enforce {
            &empty_sizes
          } else {
            &overall_min_size_reduction
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

          let r#type = v.r#type.map_or_else(
            rspack_plugin_split_chunks::create_default_module_type_filter,
            create_module_type_filter,
          );

          let layer = v.layer.map_or_else(
            rspack_plugin_split_chunks::create_default_module_layer_filter,
            create_module_layer_filter,
          );

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
            chunk_filter: v.chunks.map_or_else(
              || {
                overall_chunk_filter
                  .clone()
                  .unwrap_or_else(rspack_plugin_split_chunks::create_async_chunk_filter)
              },
              create_chunks_filter,
            ),
            min_chunks,
            min_size,
            min_size_reduction,
            automatic_name_delimiter: v
              .automatic_name_delimiter
              .unwrap_or(overall_automatic_name_delimiter.clone()),
            filename: v
              .filename
              .map(Filename::from)
              .or_else(|| overall_filename.clone()),
            reuse_existing_chunk: v.reuse_existing_chunk.unwrap_or(false),
            max_async_requests: v.max_async_requests.unwrap_or(f64::INFINITY),
            max_initial_requests: v.max_initial_requests.unwrap_or(f64::INFINITY),
            max_async_size,
            max_initial_size,
            r#type,
            layer,
            used_exports: v
              .used_exports
              .unwrap_or_else(|| raw_opts.used_exports.unwrap_or_default()),
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

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawFallbackCacheGroupOptions<'a> {
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all'")]
  #[debug(skip)]
  pub chunks: Option<Chunks<'a>>,
  pub min_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_async_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub max_initial_size: Option<Either<f64, RawSplitChunkSizes>>,
  pub automatic_name_delimiter: Option<String>,
}

fn create_module_type_filter(
  raw: Either<RspackRegex, JsString>,
) -> rspack_plugin_split_chunks::ModuleTypeFilter {
  match raw {
    Either::A(regex) => Arc::new(move |m| regex.test(m.module_type().as_str())),
    Either::B(js_str) => {
      let type_str = js_str.into_string();
      Arc::new(move |m| m.module_type().as_str() == type_str.as_str())
    }
  }
}

fn create_module_layer_filter(
  raw: Either3<RspackRegex, JsString, ThreadsafeFunction<Option<String>, bool>>,
) -> rspack_plugin_split_chunks::ModuleLayerFilter {
  match raw {
    Either3::A(regex) => Arc::new(move |layer| {
      let regex = regex.clone();
      Box::pin(async move { Ok(layer.map(|layer| regex.test(&layer)).unwrap_or_default()) })
    }),
    Either3::B(js_str) => {
      let test = js_str.into_string();
      Arc::new(move |layer| {
        let test = test.clone();
        Box::pin(async move {
          Ok(if let Some(layer) = layer {
            layer.starts_with(&test)
          } else {
            test.is_empty()
          })
        })
      })
    }
    Either3::C(f) => Arc::new(move |layer| {
      let f = f.clone();
      Box::pin(async move { f.call_with_sync(layer).await })
    }),
  }
}
