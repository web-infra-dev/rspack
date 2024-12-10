mod cacheable_context;
mod occasion;
pub mod snapshot;
pub mod storage;
use std::sync::Arc;

pub use cacheable_context::{CacheableContext, FromContext};
use occasion::MakeOccasion;
use rspack_fs::FileSystem;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use self::{
  snapshot::{Snapshot, SnapshotOptions},
  storage::{MemoryStorage, Storage, StorageOptions},
};
use super::Cache;
use crate::{make::MakeArtifact, Compilation, CompilerOptions};

#[derive(Debug, Clone)]
pub struct PersistentCacheOptions {
  pub snapshot: SnapshotOptions,
  pub storage: Vec<StorageOptions>,
}

/// Persistent cache implementation
#[derive(Debug)]
pub struct PersistentCache {
  storage: Arc<dyn Storage>,
  snapshot: Snapshot,
  make_occasion: MakeOccasion,
}

impl PersistentCache {
  pub fn new(
    option: &PersistentCacheOptions,
    input_filesystem: Arc<dyn FileSystem>,
    compiler_options: Arc<CompilerOptions>,
  ) -> Self {
    let storage = Arc::new(MemoryStorage::default());
    let context = Arc::new(CacheableContext {
      options: compiler_options,
      input_filesystem: input_filesystem.clone(),
    });
    let make_occasion = MakeOccasion::new(storage.clone(), context);
    Self {
      snapshot: Snapshot::new(option.snapshot.clone(), input_filesystem, storage.clone()),
      storage,
      make_occasion,
    }
  }
}

impl Cache for PersistentCache {
  fn before_compile(&self, compilation: &mut Compilation) {
    if compilation.modified_files.is_empty() && compilation.removed_files.is_empty() {
      // inject modified_files and removed_files
      let (modified_paths, removed_paths) = self.snapshot.calc_modified_paths();
      compilation.modified_files = modified_paths;
      compilation.removed_files = removed_paths;
    }
  }

  fn after_compile(&self, compilation: &Compilation) {
    // TODO add a all_dependencies to collect dependencies
    let (_, file_added, file_removed) = compilation.file_dependencies();
    let (_, context_added, context_removed) = compilation.context_dependencies();
    let (_, missing_added, missing_removed) = compilation.missing_dependencies();
    let (_, build_added, build_removed) = compilation.build_dependencies();
    let modified_paths: HashSet<ArcPath> = compilation
      .modified_files
      .iter()
      .chain(file_added)
      .chain(context_added)
      .chain(missing_added)
      .chain(build_added)
      .cloned()
      .collect();
    let removed_paths: HashSet<ArcPath> = compilation
      .removed_files
      .iter()
      .chain(file_removed)
      .chain(context_removed)
      .chain(missing_removed)
      .chain(build_removed)
      .cloned()
      .collect();
    self
      .snapshot
      .remove(removed_paths.iter().map(|item| item.as_ref()));
    self
      .snapshot
      .add(modified_paths.iter().map(|item| item.as_ref()));

    self.storage.trigger_save();
  }

  fn before_make(&self, make_artifact: &mut MakeArtifact) {
    if !make_artifact.initialized {
      if let Ok(artifact) = self.make_occasion.recovery() {
        *make_artifact = artifact;
      }
    }
  }

  fn after_make(&self, make_artifact: &MakeArtifact) {
    self.make_occasion.save(make_artifact);
  }
}
