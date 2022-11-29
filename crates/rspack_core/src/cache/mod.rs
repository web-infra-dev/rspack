use crate::CompilerOptions;
use std::sync::Arc;
use tokio::sync::Mutex;

mod occasion;
mod snapshot;
mod storage;
use occasion::ResolveModuleOccasion;
use snapshot::SnapshotManager;
use storage::new_storage;

#[derive(Debug)]
pub struct Cache {
  is_idle: Mutex<bool>,
  snapshot_manager: Arc<SnapshotManager>,
  pub resolve_module_occasion: ResolveModuleOccasion,
}

impl Cache {
  pub fn new(options: Arc<CompilerOptions>) -> Self {
    let snapshot_manager = Arc::new(SnapshotManager::new(options.snapshot.clone()));
    Self {
      is_idle: Mutex::new(true),
      snapshot_manager: snapshot_manager.clone(),
      resolve_module_occasion: ResolveModuleOccasion::new(
        new_storage(&options.cache),
        snapshot_manager,
      ),
    }
  }

  pub async fn begin_idle(&self) {
    let mut is_idle = self.is_idle.lock().await;
    if *is_idle {
      return;
    }
    self.snapshot_manager.clear().await;
    *is_idle = true;
  }

  pub async fn end_idle(&self) {
    let mut is_idle = self.is_idle.lock().await;
    if !*is_idle {
      return;
    }
    *is_idle = false;
  }
}
