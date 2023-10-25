use std::sync::Arc;

use derivative::Derivative;
use napi::{Either, JsString};
use napi_derive::napi;
use new_split_chunks_plugin::ModuleTypeFilter;
use rspack_core::SourceType;
use rspack_napi_shared::{JsRegExp, JsRegExpExt, JsStringExt};
use rspack_plugin_split_chunks::{CacheGroupOptions, ChunkType, SplitChunksOptions, TestFn};
use serde::Deserialize;

type Chunks = Either<JsRegExp, JsString>;

#[derive(Derivative, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[derivative(Debug)]
pub struct RawSplitChunksOptions {
  pub fallback_cache_group: Option<RawFallbackCacheGroupOptions>,
  pub name: Option<String>,
  pub cache_groups: Option<Vec<RawCacheGroupOptions>>,
  /// What kind of chunks should be selected.
  #[serde(skip_deserializing)]
  #[napi(ts_type = "RegExp | 'async' | 'initial' | 'all'")]
  #[derivative(Debug = "ignore")]
  pub chunks: Option<Chunks>,
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
      chunks: value.chunks.map(|chunks| {
        let Either::B(chunks) = chunks else {
          panic!("expected string")
        };
        let chunks = chunks.into_string();
        match chunks.as_str() {
          "initial" => ChunkType::Initial,
          "async" => ChunkType::Async,
          "all" => ChunkType::All,
          _ => panic!("Invalid chunk type: {chunks}"),
        }
      }),
      ..Default::default()
    };

    defaults
      .cache_groups
      .extend(value.cache_groups.unwrap_or_default().into_iter().map(|v| {
        (
          v.key,
          CacheGroupOptions {
            name: v.name,
            priority: v.priority,
            reuse_existing_chunk: Some(false),
            test: v.test.map(|test| {
              let test = match test {
                Either::A(s) => s.into_string(),
                Either::B(_reg) => unimplemented!(),
              };
              let f: TestFn = Arc::new(move |module| {
                let re = rspack_regex::RspackRegex::new(&test)
                  .unwrap_or_else(|_| panic!("Invalid regex: {}", &test));
                module
                  .name_for_condition()
                  .map_or(false, |name| re.test(&name))
              });
              f
            }),
            chunks: v.chunks.map(|chunks| {
              let Either::B(chunks) = chunks else {
                    panic!("expected string")
                  };
              let chunks = chunks.into_string();
              match chunks.as_str() {
                "initial" => ChunkType::Initial,
                "async" => ChunkType::Async,
                "all" => ChunkType::All,
                _ => panic!("Invalid chunk type: {chunks}"),
              }
            }),
            min_chunks: v.min_chunks,
            ..Default::default()
          },
        )
      }));
    defaults
  }
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
  #[napi(ts_type = "RegExp | string")]
  #[derivative(Debug = "ignore")]
  pub test: Option<Either<JsString, JsRegExp>>,
  //   pub filename: String,
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

fn create_chunks_filter(raw: Chunks) -> rspack_plugin_split_chunks_new::ChunkFilter {
  match raw {
    Either::A(reg) => {
      rspack_plugin_split_chunks_new::create_regex_chunk_filter_from_str(reg.to_rspack_regex())
    }
    Either::B(js_str) => {
      let js_str = js_str.into_string();
      rspack_plugin_split_chunks_new::create_chunk_filter_from_str(&js_str)
    }
  }
}

impl From<RawSplitChunksOptions> for new_split_chunks_plugin::PluginOptions {
  fn from(raw_opts: RawSplitChunksOptions) -> Self {
    use new_split_chunks_plugin::SplitChunkSizes;

    let mut cache_groups = vec![];

    let overall_chunk_filter = raw_opts.chunks.map(create_chunks_filter);

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
            .unwrap_or_else(rspack_plugin_split_chunks_new::create_default_module_type_filter);

          new_split_chunks_plugin::CacheGroup {
            id_hint: v.id_hint.unwrap_or_else(|| v.key.clone()),
            key: v.key,
            name: v
              .name
              .map(new_split_chunks_plugin::create_chunk_name_getter_by_const_name)
              .unwrap_or_else(|| overall_name_getter.clone()),
            priority: v.priority.unwrap_or(0) as f64,
            test: create_module_filter(v.test),
            chunk_filter: v.chunks.map(create_chunks_filter).unwrap_or_else(|| {
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
}

fn create_module_type_filter(raw: Either<JsRegExp, JsString>) -> ModuleTypeFilter {
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

pub fn create_module_filter(
  re: Option<Either<JsString, JsRegExp>>,
) -> new_split_chunks_plugin::ModuleFilter {
  re.map(|test| match test {
    Either::A(data) => {
      let data = data.into_string();
      rspack_plugin_split_chunks_new::create_module_filter_from_rspack_str(data)
    }
    Either::B(data) => {
      let re = data.to_rspack_regex();
      rspack_plugin_split_chunks_new::create_module_filter_from_rspack_regex(re)
    }
  })
  .unwrap_or_else(rspack_plugin_split_chunks_new::create_default_module_filter)
}
