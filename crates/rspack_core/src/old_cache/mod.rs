use std::{
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use crate::CompilerOptions;

mod local;
mod occasion;
mod storage;
pub use local::*;
use occasion::{CodeGenerateOccasion, CreateChunkAssetsOccasion};
use storage::new_storage;

#[derive(Debug)]
pub struct Cache {
  is_idle: AtomicBool,
  pub code_generate_occasion: CodeGenerateOccasion,
  pub create_chunk_assets_occasion: CreateChunkAssetsOccasion,
}

impl Cache {
  pub fn new(options: Arc<CompilerOptions>) -> Self {
    Self {
      is_idle: true.into(),
      code_generate_occasion: CodeGenerateOccasion::new(new_storage(&options.cache)),
      create_chunk_assets_occasion: CreateChunkAssetsOccasion::new(new_storage(&options.cache)),
    }
  }

  pub fn set_modified_files(&self, _modified_files: Vec<PathBuf>) {
    // TODO
  }

  pub fn begin_idle(&self) {
    if self.is_idle.load(Ordering::Relaxed) {
      // TODO clean cache
    }
  }

  pub fn end_idle(&self) {
    self.is_idle.store(false, Ordering::Relaxed);
  }
}
