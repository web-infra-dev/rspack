#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{AliasMap, CompilerOptionsBuilder, Resolve};
use rspack_plugin_split_chunks::{CacheGroupOptions, ChunkType, SizeType, SplitChunksOptions};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use crate::RawOption;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawSplitChunksOptions {
  pub cache_groups: HashMap<String, RawCacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub default_size_types: Option<Vec<SizeType>>,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawSplitChunksOptions {
  pub cache_groups: HashMap<String, RawCacheGroupOptions>,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub default_size_types: Option<Vec<SizeType>>,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
}

impl RawOption<SplitChunksOptions> for RawSplitChunksOptions {
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<SplitChunksOptions> {
    let mut defaults = SplitChunksOptions::default();
    defaults.chunks = ChunkType::All;
    defaults.cache_groups = self
      .cache_groups
      .into_iter()
      .map(|(k, v)| {
        (
          k,
          CacheGroupOptions {
            name: v.name.clone(),
            priority: 0,
            reuse_existing_chunk: false,
            r#type: SizeType::JavaScript,
            test: Arc::new(move |module| {
              let re = regex::Regex::new(&v.test).unwrap();
              re.is_match(&module.id)
            }),
            filename: v.name,
            enforce: false,
            id_hint: Default::default(),
            chunks: ChunkType::All,
            automatic_name_delimiter: "~".to_string(),
            max_async_requests: 30,
            max_initial_requests: 30,
            min_chunks: 1,
            min_size: 20000,
            min_size_reduction: 20000,
            enforce_size_threshold: 50000,
            min_remaining_size: 0,
            max_size: 0,
            max_async_size: usize::MAX,
            max_initial_size: usize::MAX,
          },
        )
      })
      .collect();
    Ok(defaults)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawSplitChunksOptions {
      cache_groups: HashMap::new(),
      chunks: None,
    }
  }
}

#[cfg(feature = "node-api")]
#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawCacheGroupOptions {
  //   pub priority: isize,
  //   pub reuse_existing_chunk: bool,
  //   pub r#type: SizeType,
  pub test: String,
  //   pub filename: String,
  //   pub enforce: bool,
  //   pub id_hint: String,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
  // TODO: supports function
  pub name: String,
  // used_exports: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawCacheGroupOptions {
  //   pub priority: isize,
  //   pub reuse_existing_chunk: bool,
  //   pub r#type: SizeType,
  pub test: String,
  //   pub filename: String,
  //   pub enforce: bool,
  //   pub id_hint: String,
  /// What kind of chunks should be selected.
  pub chunks: Option<String>,
  //   pub automatic_name_delimiter: String,
  //   pub max_async_requests: usize,
  //   pub max_initial_requests: usize,
  //   pub min_chunks: usize,
  // hide_path_info: bool,
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
  // TODO: supports function
  pub name: String,
  // used_exports: bool,
}
