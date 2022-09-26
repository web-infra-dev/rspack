#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{AliasMap, CompilerOptionsBuilder, Resolve};
use serde::Deserialize;
use std::collections::HashMap;

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
