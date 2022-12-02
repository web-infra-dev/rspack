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
  storage: Option<RwLock<Box<Storage>>>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl BuildModuleOccasion {
  pub fn new(storage: Option<Box<Storage>>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage: storage.map(RwLock::new),
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
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator(module).await,
    };

    let mut need_cache = false;
    let id: String = module.identifier().into();
    if module.as_normal_module().is_some() {
      // normal module
      // TODO cache all module type
      let storage = storage.read().await;
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
      storage.write().await.set(id, (snapshot, data.clone()));
    }
    Ok(data)
  }
}
