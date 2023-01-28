use crate::cache_group_source::SplitChunkSizes;

#[derive(Debug)]
pub struct MaxSizeQueueItem {
  pub min_size: SplitChunkSizes,
  pub max_async_size: SplitChunkSizes,
  pub max_initial_size: SplitChunkSizes,
  pub keys: Vec<String>,
}
