use std::sync::Arc;

use tokio::sync::Mutex;

use crate::CompilerOptions;

mod local;
mod occasion;
mod snapshot;
mod storage;
pub use local::*;
use occasion::{
  BuildModuleOccasion, CodeGenerateOccasion, CreateChunkAssetsOccasion, ResolveModuleOccasion,
};
use snapshot::SnapshotManager;
use storage::new_storage;

#[derive(Debug)]
pub struct Cache {
  is_idle: Mutex<bool>,
  snapshot_manager: Arc<SnapshotManager>,
  pub resolve_module_occasion: ResolveModuleOccasion,
  pub build_module_occasion: BuildModuleOccasion,
  pub code_generate_occasion: CodeGenerateOccasion,
  pub create_chunk_assets_occasion: CreateChunkAssetsOccasion,
}

impl Cache {
  pub fn new(options: Arc<CompilerOptions>) -> Self {
    let snapshot_manager = Arc::new(SnapshotManager::new(options.snapshot.clone()));
    Self {
      is_idle: Mutex::new(true),
      snapshot_manager: snapshot_manager.clone(),
      resolve_module_occasion: ResolveModuleOccasion::new(
        new_storage(&options.cache),
        snapshot_manager.clone(),
      ),
      build_module_occasion: BuildModuleOccasion::new(
        new_storage(&options.cache),
        snapshot_manager,
      ),
      code_generate_occasion: CodeGenerateOccasion::new(new_storage(&options.cache)),
      create_chunk_assets_occasion: CreateChunkAssetsOccasion::new(new_storage(&options.cache)),
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
