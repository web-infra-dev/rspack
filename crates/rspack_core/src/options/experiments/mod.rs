mod cache;

pub use cache::CacheOptions as ExperimentCacheOptions;

use crate::incremental::IncrementalPasses;

// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<Experiments> for ExperimentsBuilder` should be updated.
#[derive(Debug)]
pub struct Experiments {
  pub layers: bool,
  pub incremental: IncrementalPasses,
  pub parallel_code_splitting: bool,
  pub top_level_await: bool,
  pub rspack_future: RspackFuture,
  pub cache: ExperimentCacheOptions,
}

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug)]
pub struct RspackFuture {}
