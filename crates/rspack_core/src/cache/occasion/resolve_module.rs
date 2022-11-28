use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  ResolveArgs, ResolveResult,
};
use futures::future::BoxFuture;
use rspack_error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

type Storage = dyn storage::Storage<(Snapshot, ResolveResult)>;

// TODO consider multi thread
#[derive(Debug)]
pub struct ResolveModuleOccasion {
  storage: RwLock<Box<Storage>>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl ResolveModuleOccasion {
  pub fn new(storage: Box<Storage>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage: RwLock::new(storage),
      snapshot_manager,
    }
  }

  pub async fn use_cache<'a, F>(&self, args: ResolveArgs<'a>, generator: F) -> Result<ResolveResult>
  where
    F: Fn(ResolveArgs<'a>) -> BoxFuture<'a, Result<ResolveResult>>,
  {
    let id = format!("{:?}|{:?}", args.importer, args.span);
    {
      // read
      let storage = self.storage.read().await;
      if let Some((snapshot, data)) = storage.get(&id) {
        let valid = self
          .snapshot_manager
          .check_snapshot_valid(&snapshot)
          .await
          .unwrap_or(false);

        if valid {
          return Ok(data);
        }
      };
    }

    // run generator and save to cache
    let data = generator(args).await?;
    let snapshot = self
      .snapshot_manager
      .create_snapshot(vec![id.clone()], |option| &option.resolve)
      .await?;
    self
      .storage
      .write()
      .await
      .set(id.clone(), (snapshot, data.clone()));
    Ok(data)
  }
}
