use std::sync::{atomic::Ordering::Relaxed, Arc};

use rspack_cacheable::{cacheable, from_bytes, to_bytes};
use rspack_error::Result;

use super::Storage;
use crate::DEPENDENCY_ID;

const SCOPE: &str = "occasion_make_meta";

/// The value struct of current storage scope
#[cacheable]
pub struct Meta {
  // Ignore entry_dependencies, compile will regenerate it.
  // pub entry_dependencies: HashSet<DependencyId>,
  pub next_dependencies_id: u32,
}

#[tracing::instrument("Cache::Occasion::Make::Meta::save", skip_all)]
pub fn save_meta(storage: &Arc<dyn Storage>) {
  let meta = Meta {
    next_dependencies_id: DEPENDENCY_ID.load(Relaxed),
  };
  storage.set(
    SCOPE,
    "default".as_bytes().to_vec(),
    to_bytes(&meta, &()).expect("should to bytes success"),
  );
}

#[tracing::instrument("Cache::Occasion::Make::Meta::recovery", skip_all)]
pub async fn recovery_meta(storage: &Arc<dyn Storage>) -> Result<()> {
  let Some((_, value)) = storage.load(SCOPE).await?.pop() else {
    return Ok(());
  };
  let meta: Meta = from_bytes(&value, &()).expect("should from bytes success");
  // TODO make dependency id to string like module id
  if DEPENDENCY_ID.load(Relaxed) < meta.next_dependencies_id {
    DEPENDENCY_ID.store(meta.next_dependencies_id, Relaxed);
  }
  Ok(())
}
