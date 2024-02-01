use derivative::Derivative;
use rspack_core::Filename;

use super::cache_group_test::CacheGroupTest;
use super::chunk_name::ChunkNameGetter;
use crate::common::{ChunkFilter, ModuleTypeFilter, SplitChunkSizes};

#[derive(Derivative)]
#[derivative(Debug)]
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
  #[derivative(Debug = "ignore")]
  pub chunk_filter: ChunkFilter,
  #[derivative(Debug = "ignore")]
  pub test: CacheGroupTest,
  #[derivative(Debug = "ignore")]
  pub r#type: ModuleTypeFilter,
  /// `name` is used to create chunk
  #[derivative(Debug = "ignore")]
  pub name: ChunkNameGetter,
  pub priority: f64,
  pub min_size: SplitChunkSizes,
  pub reuse_existing_chunk: bool,
  /// number of referenced chunks
  pub min_chunks: u32,
  pub id_hint: String,
  pub max_initial_requests: u32,
  pub max_async_requests: u32,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub filename: Option<Filename>,
  pub automatic_name_delimiter: String,
}
