use crate::CompilerOptions;
use std::sync::Arc;

mod occasion;
mod snapshot;
mod storage;
use occasion::ResolveModuleOccasion;
use snapshot::SnapshotManager;
use storage::new_storage;

#[derive(Debug)]
pub struct Cache {
  pub snapshot_manager: Arc<SnapshotManager>,
  pub resolve_module_occasion: ResolveModuleOccasion,
}

impl Cache {
  pub fn new(options: Arc<CompilerOptions>) -> Self {
    let snapshot_manager = Arc::new(SnapshotManager::new(options.snapshot.clone()));
    Self {
      snapshot_manager: snapshot_manager.clone(),
      resolve_module_occasion: ResolveModuleOccasion::new(
        new_storage(&options.cache),
        snapshot_manager,
      ),
    }
  }

  pub fn begin_idle(&self) {}

  pub async fn end_idle(&self) {
    self.snapshot_manager.clear().await;
  }
}
