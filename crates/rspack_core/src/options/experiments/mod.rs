mod cache;

pub use cache::CacheOptions as ExperimentCacheOptions;

use crate::incremental::IncrementalOptions;

// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<Experiments> for ExperimentsBuilder` should be updated.
#[derive(Debug)]
pub struct Experiments {
  pub layers: bool,
  pub incremental: IncrementalOptions,
  pub parallel_code_splitting: bool,
  pub rspack_future: RspackFuture,
  pub cache: ExperimentCacheOptions,
  pub inline_const: bool,
  pub inline_enum: bool,
  pub type_reexports_presence: bool,
  pub lazy_barrel: bool,
}

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug)]
pub struct RspackFuture {}
