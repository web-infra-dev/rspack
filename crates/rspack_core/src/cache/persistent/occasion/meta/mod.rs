use std::sync::Arc;

use rspack_cacheable::{cacheable, from_bytes, to_bytes};
use rspack_error::Result;
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::super::Storage;

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
}

impl MetaOccasion {
  pub fn new(storage: Arc<dyn Storage>) -> Self {
    Self { storage }
  }

  #[tracing::instrument("Cache::Occasion::Meta::save", skip_all)]
  pub fn save(&self) {
    let meta = Meta {
      max_dependencies_id: get_current_dependency_id(),
    };
    self.storage.set(
      SCOPE,
      "default".as_bytes().to_vec(),
      to_bytes(&meta, &()).expect("should to bytes success"),
    );
  }

  #[tracing::instrument("Cache::Occasion::Meta::recovery", skip_all)]
  pub async fn recovery(&self) -> Result<()> {
    let Some((_, value)) = self.storage.load(SCOPE).await?.pop() else {
      return Ok(());
    };

    let meta: Meta = from_bytes(&value, &()).expect("should from bytes success");
    if get_current_dependency_id() != 0 {
      panic!("The global dependency id generator is not 0 when the persistent cache is restored.");
    }
    set_current_dependency_id(meta.max_dependencies_id);
    Ok(())
  }
}
