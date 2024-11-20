use super::Cache;
use crate::make::MakeArtifact;

/// Disable cache implementation
///
/// Disable cache will clean the corresponding artifact before target step run.
#[derive(Debug)]
pub struct DisableCache;

impl Cache for DisableCache {
  fn before_make(&self, make_artifact: &mut MakeArtifact) {
    *make_artifact = Default::default();
  }
}
