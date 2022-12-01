use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  BoxModule, BuildResult,
};
use futures::future::BoxFuture;
use rspack_error::{Result, TWithDiagnosticArray};
use std::sync::Arc;
use tokio::sync::RwLock;

type Storage = dyn storage::Storage<(Snapshot, TWithDiagnosticArray<BuildResult>)>;

#[derive(Debug)]
pub struct BuildModuleOccasion {
  storage: RwLock<Box<Storage>>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl BuildModuleOccasion {
  pub fn new(storage: Box<Storage>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage: RwLock::new(storage),
      snapshot_manager,
    }
  }

  pub async fn use_cache<'a, F>(
    &self,
    module: &'a mut BoxModule,
    generator: F,
  ) -> Result<TWithDiagnosticArray<BuildResult>>
  where
    F: Fn(&'a mut BoxModule) -> BoxFuture<'a, Result<TWithDiagnosticArray<BuildResult>>>,
  {
    let mut need_cache = false;
    let id = module.identifier().as_ref().to_string();
    if module.as_normal_module().is_some() {
      // normal module
      // TODO cache all module type
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
      need_cache = true;
    }

    // run generator and save to cache
    let data = generator(module).await?;
    if need_cache && data.inner.cacheable {
      let snapshot = self
        .snapshot_manager
        // TODO replace id with source file path or just cache normal module
        .create_snapshot(vec![id.clone()], |option| &option.module)
        .await?;
      self
        .storage
        .write()
        .await
        .set(id.clone(), (snapshot, data.clone()));
    }
    Ok(data)
  }
}
