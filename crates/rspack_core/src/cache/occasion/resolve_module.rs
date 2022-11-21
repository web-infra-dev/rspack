use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  ResolveArgs, ResolveResult,
};
use rspack_error::Result;
use std::sync::Arc;

type Storage = dyn storage::Storage<(Snapshot, ResolveResult)>;

// TODO consider multi thread
#[derive(Debug)]
pub struct ResolveModuleOccasion {
  storage: Box<Storage>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl ResolveModuleOccasion {
  pub fn new(storage: Box<Storage>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage,
      snapshot_manager,
    }
  }

  pub async fn use_cache<F>(&mut self, args: ResolveArgs<'_>, generator: F) -> Result<ResolveResult>
  where
    F: FnOnce(ResolveArgs) -> Result<ResolveResult>,
  {
    let id = format!("{:?}|{:?}", args.importer, args.span);
    if let Some((snapshot, data)) = self.storage.get(&id) {
      let valid = self
        .snapshot_manager
        .check_snapshot_valid(&snapshot)
        .await
        .unwrap_or(false);

      if valid {
        return Ok(data);
      }
    };

    // run generator and save to cache
    let data = generator(args)?;
    let snapshot = self
      .snapshot_manager
      .create_snapshot(vec![id.clone()], |option| &option.resolve)
      .await?;
    self.storage.set(id.clone(), (snapshot, data.clone()));
    Ok(data)
  }
}
