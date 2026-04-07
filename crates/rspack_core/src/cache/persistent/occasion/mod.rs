pub mod make;
pub mod meta;

pub use make::MakeOccasion;
pub use meta::MetaOccasion;
use rspack_error::Result;

use super::storage::Storage;

/// An `Occasion` represents a distinct phase of the persistent cache lifecycle.
///
/// Each occasion owns one storage scope and is responsible for:
/// - serialising its artifact into storage (`save`)
/// - deserialising its artifact from storage (`recovery`)
/// - clearing its scope when the cached data is stale (`reset`)
///
/// `BuildDeps` and `Snapshot` are not occasions: they operate across multiple
/// scopes and have more complex lifecycle semantics.
#[async_trait::async_trait]
pub trait Occasion {
  /// The data produced/consumed by this occasion.
  type Artifact: Send;

  /// Clear this occasion's scope in storage.
  fn reset(&self, storage: &mut dyn Storage);

  /// Persist `artifact` into storage.  Only called when not in readonly mode.
  fn save(&self, storage: &mut dyn Storage, artifact: &Self::Artifact);

  /// Load and reconstruct the artifact from storage.
  async fn recovery(&self, storage: &dyn Storage) -> Result<Self::Artifact>;
}
