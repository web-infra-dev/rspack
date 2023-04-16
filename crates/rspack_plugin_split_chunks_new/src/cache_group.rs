use derivative::Derivative;

use crate::common::{ChunkFilter, ModuleFilter};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CacheGroup {
  #[derivative(Debug = "ignore")]
  pub chunk_filter: ChunkFilter,
  #[derivative(Debug = "ignore")]
  pub test: ModuleFilter,
  pub name: String,
  pub priority: f64,
  pub min_chunks: u32,
  // pub max_size: f64,
  // pub min_size: f64,
}
