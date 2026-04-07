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

use self::{
  raw_split_chunk_cache_group_test::{
    RawCacheGroupTest, default_cache_group_test, normalize_raw_cache_group_test,
  },
  raw_split_chunk_chunks::{Chunks, create_chunks_filter},
  raw_split_chunk_name::default_chunk_option_name,
  raw_split_chunk_size::RawSplitChunkSizes,
};
use crate::{filename::JsFilename, js_regex::JsRegExp};

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
  pub r#type: Option<Either<JsRegExp, JsString<'a>>>,
  #[napi(ts_type = "RegExp | string | ((layer?: string) => boolean)")]
  #[debug(skip)]
  pub layer: Option<Either3<JsRegExp, JsString<'a>, ThreadsafeFunction<Option<String>, bool>>>,
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

impl<'a> TryFrom<RawSplitChunksOptions<'a>> for rspack_plugin_split_chunks::PluginOptions {
  type Error = rspack_error::Error;

  fn try_from(raw_opts: RawSplitChunksOptions) -> Result<Self, Self::Error> {
    use rspack_plugin_split_chunks::SplitChunkSizes;

    let RawSplitChunksOptions {
      fallback_cache_group,
      name,
      filename,
      cache_groups,
      chunks,
      used_exports,
      automatic_name_delimiter,
      default_size_types,
      min_chunks,
      hide_path_info,
      min_size,
      min_size_reduction,
      max_size,
      max_async_size,
      max_initial_size,
      ..
    } = raw_opts;

    let overall_filename = filename.map(Filename::from);
    let overall_chunk_filter = chunks.map(create_chunks_filter).transpose()?;
    let overall_min_chunks = min_chunks.unwrap_or(1);
    let overall_used_exports = used_exports.unwrap_or_default();
    let overall_name_getter = name.map_or_else(default_chunk_option_name, normalize_raw_chunk_name);

    let default_size_types = default_size_types
      .into_iter()
      .map(|size_type: String| SourceType::from(size_type.as_str()))
      .collect::<Vec<_>>();

    let create_sizes = |size: Option<Either<f64, RawSplitChunkSizes>>| match size {
      Some(Either::A(size)) => SplitChunkSizes::with_initial_value(&default_size_types, size),
      Some(Either::B(sizes)) => sizes.into(),
      None => SplitChunkSizes::default(),
    };

    let empty_sizes = SplitChunkSizes::empty();

    let overall_min_size = create_sizes(min_size);
    let overall_min_size_reduction = create_sizes(min_size_reduction);
    let overall_max_size = create_sizes(max_size);
    let overall_max_async_size = create_sizes(max_async_size).merge(&overall_max_size);
    let overall_max_initial_size = create_sizes(max_initial_size).merge(&overall_max_size);
    let overall_automatic_name_delimiter =
      automatic_name_delimiter.unwrap_or(DEFAULT_DELIMITER.to_string());

    let default_cache_group_test = default_cache_group_test();
    let default_module_type_filter =
      rspack_plugin_split_chunks::create_default_module_type_filter();
    let default_module_layer_filter =
      rspack_plugin_split_chunks::create_default_module_layer_filter();
    let default_group_chunk_filter = overall_chunk_filter
      .clone()
      .unwrap_or_else(rspack_plugin_split_chunks::create_async_chunk_filter);

    let raw_cache_groups = cache_groups.unwrap_or_default();
    let mut cache_groups = Vec::with_capacity(raw_cache_groups.len());
    for cache_group in raw_cache_groups {
      let RawCacheGroupOptions {
        key,
        priority,
        test,
        filename,
        id_hint,
        chunks,
        r#type,
        layer,
        automatic_name_delimiter,
        min_chunks,
        min_size,
        min_size_reduction,
        max_size,
        max_async_size,
        max_initial_size,
        max_async_requests,
        max_initial_requests,
        name,
        reuse_existing_chunk,
        enforce,
        used_exports,
      } = cache_group;

      let enforce = enforce.unwrap_or_default();
      let min_size = create_sizes(min_size).merge(if enforce {
        &empty_sizes
      } else {
        &overall_min_size
      });
      let min_size_reduction = create_sizes(min_size_reduction).merge(if enforce {
        &empty_sizes
      } else {
        &overall_min_size_reduction
      });

      let max_size = create_sizes(max_size);
      let max_async_size = create_sizes(max_async_size)
        .merge(&max_size)
        .merge(if enforce {
          &empty_sizes
        } else {
          &overall_max_async_size
        });
      let max_initial_size = create_sizes(max_initial_size)
        .merge(&max_size)
        .merge(if enforce {
          &empty_sizes
        } else {
          &overall_max_initial_size
        });

      let min_chunks = min_chunks.unwrap_or(if enforce { 1 } else { overall_min_chunks });
      let r#type = match r#type {
        Some(raw) => create_module_type_filter(raw)?,
        None => default_module_type_filter.clone(),
      };
      let layer = match layer {
        Some(raw) => create_module_layer_filter(raw)?,
        None => default_module_layer_filter.clone(),
      };

      let mut name = name.map_or_else(default_chunk_option_name, normalize_raw_chunk_name);
      if matches!(name, ChunkNameGetter::Disabled) {
        name = overall_name_getter.clone();
      }

      let test = match test {
        Some(raw) => normalize_raw_cache_group_test(raw)?,
        None => default_cache_group_test.clone(),
      };
      let chunk_filter = match chunks {
        Some(raw) => create_chunks_filter(raw)?,
        None => default_group_chunk_filter.clone(),
      };

      cache_groups.push(rspack_plugin_split_chunks::CacheGroup {
        id_hint: id_hint.unwrap_or_else(|| key.clone()),
        key,
        name,
        priority: priority.unwrap_or(0) as f64,
        test,
        chunk_filter,
        min_chunks,
        min_size,
        min_size_reduction,
        automatic_name_delimiter: automatic_name_delimiter
          .unwrap_or_else(|| overall_automatic_name_delimiter.clone()),
        filename: filename
          .map(Filename::from)
          .or_else(|| overall_filename.clone()),
        reuse_existing_chunk: reuse_existing_chunk.unwrap_or(false),
        max_async_requests: max_async_requests.unwrap_or(f64::INFINITY),
        max_initial_requests: max_initial_requests.unwrap_or(f64::INFINITY),
        max_async_size,
        max_initial_size,
        r#type,
        layer,
        used_exports: used_exports.unwrap_or(overall_used_exports),
      });
    }

    let raw_fallback_cache_group = fallback_cache_group.unwrap_or_default();

    let fallback_chunks_filter = raw_fallback_cache_group
      .chunks
      .map(create_chunks_filter)
      .transpose()?;

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

    let default_fallback_chunk_filter =
      overall_chunk_filter.unwrap_or_else(rspack_plugin_split_chunks::create_all_chunk_filter);

    Ok(rspack_plugin_split_chunks::PluginOptions {
      cache_groups,
      fallback_cache_group: rspack_plugin_split_chunks::FallbackCacheGroup {
        chunks_filter: fallback_chunks_filter.unwrap_or(default_fallback_chunk_filter),
        min_size: fallback_min_size,
        max_async_size: fallback_max_async_size,
        max_initial_size: fallback_max_initial_size,
        automatic_name_delimiter: raw_fallback_cache_group
          .automatic_name_delimiter
          .unwrap_or(overall_automatic_name_delimiter.clone()),
      },
      hide_path_info,
    })
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
  raw: Either<JsRegExp, JsString>,
) -> rspack_error::Result<rspack_plugin_split_chunks::ModuleTypeFilter> {
  Ok(match raw {
    Either::A(regex) => {
      let regex = rspack_regex::RspackRegex::try_from(regex)?;
      Arc::new(move |m| regex.test(m.module_type().as_str()))
    }
    Either::B(js_str) => {
      let type_str = js_str.into_string();
      Arc::new(move |m| m.module_type().as_str() == type_str.as_str())
    }
  })
}

fn create_module_layer_filter(
  raw: Either3<JsRegExp, JsString, ThreadsafeFunction<Option<String>, bool>>,
) -> rspack_error::Result<rspack_plugin_split_chunks::ModuleLayerFilter> {
  Ok(match raw {
    Either3::A(regex) => {
      let regex = rspack_regex::RspackRegex::try_from(regex)?;
      Arc::new(move |layer| {
        let regex = regex.clone();
        Box::pin(async move { Ok(layer.map(|layer| regex.test(&layer)).unwrap_or_default()) })
      })
    }
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
  })
}
