pub struct CacheGroupSource {
  pub key: String,
  pub priority: usize,
  pub get_name: Box<dyn Fn() -> String>,
  pub chunks_filter: Box<dyn Fn() -> bool>,
  pub enforce: bool,
  pub min_chunks: usize,
  pub max_async_requests: usize,
  pub max_initial_requests: usize,
  pub filename: String,
  pub id_hint: String,
  pub automatic_name_delimiter: String,
  pub reuse_existing_chunk: bool,
  pub used_exports: bool,
  pub min_size: usize,
  pub min_size_reduction: usize,
  pub min_remaining_size: usize,
  pub enforce_size_threshold: usize,
  pub max_async_size: usize,
  pub max_initial_size: usize,
}

#[derive(Debug)]
pub struct SplitChunksOptions {}
