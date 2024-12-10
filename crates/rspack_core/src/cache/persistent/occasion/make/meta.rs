use std::sync::{atomic::Ordering::Relaxed, Arc};

use rspack_cacheable::{
  cacheable, from_bytes, to_bytes, with::Inline, DeserializeError, SerializeError,
};
use rspack_collections::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;

use super::Storage;
use crate::{BuildDependency, DEPENDENCY_ID};

const SCOPE: &str = "occasion_make_meta";

/// The value struct of current storage scope
#[cacheable]
pub struct Meta {
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: IdentifierSet,
  // Ignore entry_dependencies, compile will regenerate it.
  // pub entry_dependencies: HashSet<DependencyId>,
  pub next_dependencies_id: u32,
}

#[cacheable(as=Meta)]
pub struct MetaRef<'a> {
  #[cacheable(with=Inline)]
  pub make_failed_dependencies: &'a HashSet<BuildDependency>,
  #[cacheable(with=Inline)]
  pub make_failed_module: &'a IdentifierSet,
  pub next_dependencies_id: u32,
}

pub fn save_meta(
  make_failed_dependencies: &HashSet<BuildDependency>,
  make_failed_module: &IdentifierSet,
  storage: &Arc<dyn Storage>,
) -> Result<(), SerializeError> {
  let meta = MetaRef {
    make_failed_dependencies,
    make_failed_module,
    next_dependencies_id: DEPENDENCY_ID.load(Relaxed),
  };
  storage.set(SCOPE, "default".as_bytes().to_vec(), to_bytes(&meta, &())?);
  Ok(())
}

pub async fn recovery_meta(
  storage: &Arc<dyn Storage>,
) -> Result<(HashSet<BuildDependency>, IdentifierSet), DeserializeError> {
  let Some((_, value)) = storage.load(SCOPE).await.unwrap_or_default().pop() else {
    return Err(DeserializeError::MessageError("can not get meta data"));
  };
  let meta: Meta = from_bytes(&value, &())?;
  // TODO make dependency id to string like module id
  if DEPENDENCY_ID.load(Relaxed) < meta.next_dependencies_id {
    DEPENDENCY_ID.store(meta.next_dependencies_id, Relaxed);
  }
  Ok((meta.make_failed_dependencies, meta.make_failed_module))
}
