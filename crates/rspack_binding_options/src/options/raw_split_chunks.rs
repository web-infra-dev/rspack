#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, ModuleType};
use rspack_plugin_split_chunks::{CacheGroupOptions, ChunkType, SplitChunksOptions, TestFn};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use crate::RawOption;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawSplitChunksOptions {
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
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawSplitChunksOptions {
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
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
}

impl RawOption<SplitChunksOptions> for RawSplitChunksOptions {
  #[allow(clippy::field_reassign_with_default)]
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<SplitChunksOptions> {
    let mut defaults = SplitChunksOptions::default();

    defaults
      .cache_groups
      .extend(
        self
          .cache_groups
          .unwrap_or_default()
          .into_iter()
          .map(|(k, v)| {
            (
              k,
              CacheGroupOptions {
                name: v.name.clone(),
                priority: 0.into(),
                reuse_existing_chunk: false.into(),
                r#type: Some(ModuleType::Js),
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
                filename: v.name,
                enforce: false.into(),
                id_hint: Default::default(),
                chunks: ChunkType::All.into(),
                automatic_name_delimiter: "~".to_string().into(),
                max_async_requests: 30.into(),
                max_initial_requests: 30.into(),
                min_chunks: 1.into(),
                min_size: 20000f64.into(),
                min_size_reduction: 20000f64.into(),
                enforce_size_threshold: 50000f64.into(),
                min_remaining_size: 0f64.into(),
                max_size: 0f64.into(),
                max_async_size: f64::MAX.into(),
                max_initial_size: f64::MAX.into(),
              },
            )
          }),
      );
    Ok(defaults)
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    RawSplitChunksOptions {
      cache_groups: None,
      chunks: None,
      max_async_requests: None,
      max_initial_requests: None,
      min_chunks: None,
      min_size: None,
      enforce_size_threshold: None,
      min_remaining_size: None,
    }
  }
}

#[cfg(feature = "node-api")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCacheGroupOptions {
  pub priority: Option<i32>,
  //   pub reuse_existing_chunk: bool,
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
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
  pub name: Option<String>,
  // used_exports: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawCacheGroupOptions {
  pub priority: Option<i32>,
  //   pub reuse_existing_chunk: bool,
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
  //   pub min_size: usize,
  //   pub min_size_reduction: usize,
  //   pub enforce_size_threshold: usize,
  //   pub min_remaining_size: usize,
  // layer: String,
  //   pub max_size: usize,
  //   pub max_async_size: usize,
  //   pub max_initial_size: usize,
  pub name: Option<String>,
  // used_exports: bool,
}
