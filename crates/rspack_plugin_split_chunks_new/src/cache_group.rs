use derivative::Derivative;

use crate::common::{ChunkFilter, ModuleFilter};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CacheGroup {
  #[derivative(Debug = "ignore")]
  pub chunk_filter: ChunkFilter,
  #[derivative(Debug = "ignore")]
  pub test: ModuleFilter,
  /// `name` is used to create chunk
  pub name: String,
  pub priority: f64,
  /// number of referenced chunks
  pub min_chunks: u32,
}
