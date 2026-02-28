use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::super::{Storage, codec::CacheCodec};

pub const SCOPE: &str = "meta";

/// The meta data.
#[cacheable]
struct Meta {
  pub max_dependencies_id: u32,
}

/// Meta Occasion is used to save compiler state.
#[derive(Debug)]
pub struct MetaOccasion {
  storage: Arc<dyn Storage>,
  codec: Arc<CacheCodec>,
}

impl MetaOccasion {
  pub fn new(storage: Arc<dyn Storage>, codec: Arc<CacheCodec>) -> Self {
    Self { storage, codec }
  }

  #[tracing::instrument("Cache::Occasion::Meta::save", skip_all)]
  pub fn save(&self) {
    let meta = Meta {
      max_dependencies_id: get_current_dependency_id(),
    };
    self.storage.set(
      SCOPE,
      "default".as_bytes().to_vec(),
      self.codec.encode(&meta).expect("should encode success"),
    );
  }

  #[tracing::instrument("Cache::Occasion::Meta::recovery", skip_all)]
  pub async fn recovery(&self) -> Result<()> {
    let Some((_, value)) = self.storage.load(SCOPE).await?.pop() else {
      return Ok(());
    };

    let meta: Meta = self.codec.decode(&value).expect("should decode success");
    if get_current_dependency_id() != 0 {
      panic!("The global dependency id generator is not 0 when the persistent cache is restored.");
    }
    set_current_dependency_id(meta.max_dependencies_id);
    Ok(())
  }
}
