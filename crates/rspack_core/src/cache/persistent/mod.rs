mod cacheable_context;
mod occasion;
pub mod snapshot;
pub mod storage;
mod version;
use std::{hash::Hash, path::PathBuf, sync::Arc};

pub use cacheable_context::{CacheableContext, FromContext};
use occasion::MakeOccasion;
use rspack_error::Result;
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_macros::rspack_version;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use self::{
  snapshot::{Snapshot, SnapshotOptions},
  storage::{create_storage, Storage, StorageOptions},
};
use super::Cache;
use crate::{
  make::{MakeArtifact, MakeArtifactState},
  Compilation, CompilerOptions,
};

#[derive(Debug, Clone)]
pub struct PersistentCacheOptions {
  pub build_dependencies: Vec<PathBuf>,
  pub version: String,
  pub snapshot: SnapshotOptions,
  pub storage: StorageOptions,
}

/// Persistent cache implementation
#[derive(Debug)]
pub struct PersistentCache {
  storage: Arc<dyn Storage>,
  snapshot: Snapshot,
  make_occasion: MakeOccasion,
  async_mode: bool,
}

impl PersistentCache {
  pub fn new(
    compiler_path: &str,
    option: &PersistentCacheOptions,
    compiler_options: Arc<CompilerOptions>,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
  ) -> Self {
    let async_mode = compiler_options.mode.is_development();
    let version = version::get_version(
      input_filesystem.clone(),
      &option.build_dependencies,
      |hasher| {
        compiler_path.hash(hasher);
        option.version.hash(hasher);
        rspack_version!().hash(hasher);
        compiler_options.name.hash(hasher);
        compiler_options.mode.hash(hasher);
      },
    );
    let storage = create_storage(option.storage.clone(), version, intermediate_filesystem);
    let context = Arc::new(CacheableContext {
      options: compiler_options,
      input_filesystem: input_filesystem.clone(),
    });
    let make_occasion = MakeOccasion::new(storage.clone(), context);
    Self {
      snapshot: Snapshot::new(option.snapshot.clone(), input_filesystem, storage.clone()),
      storage,
      make_occasion,
      async_mode,
    }
  }
}

#[async_trait::async_trait]
impl Cache for PersistentCache {
  async fn before_compile(&self, compilation: &mut Compilation) -> Result<()> {
    // rebuild will pass modified_files and removed_files from js side,
    // so only calculate them when build.
    if !compilation.is_rebuild {
      let (modified_paths, removed_paths) = self.snapshot.calc_modified_paths().await?;
      tracing::info!("cache::snapshot recovery {modified_paths:?} {removed_paths:?}",);
      compilation.modified_files.extend(modified_paths);
      compilation.removed_files.extend(removed_paths);
    }
    Ok(())
  }

  async fn after_compile(&self, compilation: &Compilation) -> Result<()> {
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
      .add(modified_paths.iter().map(|item| item.as_ref()))
      .await;

    let rx = self.storage.trigger_save()?;
    if self.async_mode {
      tokio::spawn(async {
        if let Err(err) = rx.await.expect("should receive message") {
          // TODO use infra structure logger to println
          println!("persistent cache save failed. {err}");
        }
      });
    } else {
      rx.await.expect("should receive message")?;
    }

    Ok(())
  }

  async fn before_make(&self, make_artifact: &mut MakeArtifact) -> Result<()> {
    // TODO When does not need to pass variables through make_artifact.state, use compilation.is_rebuild to check
    if matches!(make_artifact.state, MakeArtifactState::Uninitialized(..)) {
      *make_artifact = self.make_occasion.recovery().await?;
    }
    Ok(())
  }

  async fn after_make(&self, make_artifact: &MakeArtifact) -> Result<()> {
    self.make_occasion.save(make_artifact);
    Ok(())
  }
}
