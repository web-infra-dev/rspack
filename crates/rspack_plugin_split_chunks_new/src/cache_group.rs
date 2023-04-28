use derivative::Derivative;

use crate::common::{ChunkFilter, ChunkNameGetter, ModuleFilter, SplitChunkSizes};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CacheGroup {
  #[derivative(Debug = "ignore")]
  pub chunk_filter: ChunkFilter,
  #[derivative(Debug = "ignore")]
  pub test: ModuleFilter,
  /// `name` is used to create chunk
  #[derivative(Debug = "ignore")]
  pub name: ChunkNameGetter,
  pub priority: f64,
  pub min_size: SplitChunkSizes,
  /// number of referenced chunks
  pub min_chunks: u32,
}
