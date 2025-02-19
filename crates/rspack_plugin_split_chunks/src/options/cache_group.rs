use derive_more::Debug;
use rspack_core::Filename;

use super::cache_group_test::CacheGroupTest;
use super::chunk_name::ChunkNameGetter;
use crate::common::{ChunkFilter, ModuleLayerFilter, ModuleTypeFilter, SplitChunkSizes};

#[derive(Debug)]
pub struct CacheGroup {
  /// For `splitChunks.cacheGroups` config
  /// ```js
  /// splitChunks: {
  ///   hello: {
  ///     test: /hello-world\.js/,
  ///     name: 'hello-world',
  ///   }
  /// }
  /// ```
  /// `hello` is the `key` here
  pub key: String,
  #[debug(skip)]
  pub chunk_filter: ChunkFilter,
  #[debug(skip)]
  pub test: CacheGroupTest,
  #[debug(skip)]
  pub r#type: ModuleTypeFilter,
  #[debug(skip)]
  pub layer: ModuleLayerFilter,
  /// `name` is used to create chunk
  #[debug(skip)]
  pub name: ChunkNameGetter,
  pub priority: f64,
  pub min_size: SplitChunkSizes,
  pub min_size_reduction: SplitChunkSizes,
  pub reuse_existing_chunk: bool,
  /// number of referenced chunks
  pub min_chunks: u32,
  pub id_hint: String,
  pub max_initial_requests: f64, // f64 for compat js Infinity
  pub max_async_requests: f64,   // f64 for compat js Infinity
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub filename: Option<Filename>,
  pub automatic_name_delimiter: String,
  pub used_exports: bool,
}
