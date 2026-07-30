use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::{Result, error};
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::{
  super::{codec::CacheCodec, snapshot::SnapshotScope, storage::Storage},
  Occasion,
};

pub const SCOPE: &str = "meta";

/// The meta data.
#[cacheable]
struct Meta {
  pub version: String,
  pub max_dependencies_id: u32,
}

/// Meta Occasion is used to save compiler state.
#[derive(Debug)]
pub struct MetaOccasion {
  codec: Arc<CacheCodec>,
  version: String,
}

impl MetaOccasion {
  pub fn new(codec: Arc<CacheCodec>, version: String) -> Self {
    Self { codec, version }
  }
}

impl Occasion for MetaOccasion {
  /// Meta has no structured artifact: it reads/writes a single global counter.
  type Artifact = ();

  fn name(&self) -> &'static str {
    "meta"
  }

  #[tracing::instrument("Cache::Occasion::Meta::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument("Cache::Occasion::Meta::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, _artifact: &()) {
    let meta = Meta {
      version: self.version.clone(),
      max_dependencies_id: get_current_dependency_id(),
    };
    storage.set(
      SCOPE,
      "default".as_bytes().to_vec(),
      self.codec.encode(&meta).expect("should encode success"),
    );
  }

  #[tracing::instrument("Cache::Occasion::Meta::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<()> {
    let scopes = storage.scopes().await?;
    let Some((_, value)) = storage.load(SCOPE).await?.pop() else {
      if scopes
        .iter()
        .any(|scope| scope != SnapshotScope::BUILD.name())
      {
        return Err(error!("persistent cache version is missing"));
      }
      return Ok(());
    };

    let meta: Meta = self.codec.decode(&value)?;
    if meta.version != self.version {
      return Err(error!("persistent cache version does not match"));
    }
    if get_current_dependency_id() != 0 {
      panic!("The global dependency id generator is not 0 when the persistent cache is restored.");
    }
    set_current_dependency_id(meta.max_dependencies_id);
    Ok(())
  }
}
