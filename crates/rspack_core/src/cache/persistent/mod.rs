pub mod build_dependencies;
mod cacheable_context;
pub mod occasion;
pub mod snapshot;
pub mod storage;

use std::{
  hash::{DefaultHasher, Hash, Hasher},
  sync::Arc,
};

pub use cacheable_context::CacheableContext;
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_paths::ArcPathSet;
use rspack_workspace::rspack_pkg_version;

use self::{
  build_dependencies::{BuildDeps, BuildDepsOptions},
  occasion::{MakeOccasion, MetaOccasion},
  snapshot::{Snapshot, SnapshotOptions},
  storage::{Storage, StorageOptions, create_storage},
};
use super::Cache;
use crate::{
  Compilation, CompilerOptions, Logger,
  compilation::build_module_graph::{BuildModuleGraphArtifact, BuildModuleGraphArtifactState},
};

#[derive(Debug, Clone, Hash)]
pub struct PersistentCacheOptions {
  pub build_dependencies: BuildDepsOptions,
  pub version: String,
  pub snapshot: SnapshotOptions,
  pub storage: StorageOptions,
}

/// Persistent cache implementation
#[derive(Debug)]
pub struct PersistentCache {
  initialized: bool,
  build_deps: BuildDeps,
  snapshot: Snapshot,
  storage: Arc<dyn Storage>,
  make_occasion: MakeOccasion,
  meta_occasion: MetaOccasion,
  async_mode: bool,
  // TODO replace to logger and output warnings directly.
  warnings: Vec<String>,
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
    let version = {
      let mut hasher = DefaultHasher::new();
      compiler_path.hash(&mut hasher);
      option.hash(&mut hasher);
      rspack_pkg_version!().hash(&mut hasher);
      compiler_options.name.hash(&mut hasher);
      compiler_options.mode.hash(&mut hasher);
      hex::encode(hasher.finish().to_ne_bytes())
    };
    let storage = create_storage(option.storage.clone(), version, intermediate_filesystem);
    let context = Arc::new(CacheableContext {});
    let make_occasion = MakeOccasion::new(storage.clone(), context);
    let meta_occasion = MetaOccasion::new(storage.clone());
    Self {
      initialized: false,
      build_deps: BuildDeps::new(
        &option.build_dependencies,
        &option.snapshot,
        input_filesystem.clone(),
        storage.clone(),
      ),
      snapshot: Snapshot::new(option.snapshot.clone(), input_filesystem, storage.clone()),
      storage,
      make_occasion,
      meta_occasion,
      async_mode,
      warnings: Default::default(),
    }
  }

  async fn initialize(&mut self) {
    if self.initialized {
      return;
    }
    self.initialized = true;

    if let Err(err) = self.build_deps.validate().await {
      self.warnings.push(err.to_string());
    }
    if let Err(err) = self.meta_occasion.recovery().await {
      self.warnings.push(err.to_string());
    }
  }

  async fn save(&mut self) {
    let rx = match self.storage.trigger_save() {
      Ok(rx) => rx,
      Err(err) => {
        self.warnings.push(err.to_string());
        return;
      }
    };
    if self.async_mode {
      tokio::spawn(async {
        if let Err(err) = rx.await.expect("should receive message") {
          // TODO use infra structure logger to println
          println!("persistent cache save failed. {err}");
        }
      });
    } else if let Err(err) = rx.await.expect("should receive message") {
      self.warnings.push(err.to_string());
    }
  }
}

#[async_trait::async_trait]
impl Cache for PersistentCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    self.initialize().await;

    // rebuild will pass modified_files and removed_files from js side,
    // so only calculate them when build.
    if !compilation.is_rebuild {
      let (is_hot_start, modified_paths, removed_paths, _) =
        match self.snapshot.calc_modified_paths().await {
          Ok(res) => res,
          Err(err) => {
            self.warnings.push(err.to_string());
            return false;
          }
        };
      tracing::debug!("cache::snapshot recovery {modified_paths:?} {removed_paths:?}",);
      compilation.modified_files.extend(modified_paths);
      compilation.removed_files.extend(removed_paths);
      return is_hot_start;
    }
    false
  }

  async fn after_compile(&mut self, compilation: &Compilation) {
    // save meta
    self.meta_occasion.save();

    // save snapshot
    // TODO add a all_dependencies to collect dependencies
    let (_, file_added, file_removed) = compilation.file_dependencies();
    let (_, context_added, context_removed) = compilation.context_dependencies();
    let (_, missing_added, missing_removed) = compilation.missing_dependencies();
    let (_, build_added, _) = compilation.build_dependencies();
    let modified_paths: ArcPathSet = compilation
      .modified_files
      .iter()
      .chain(file_added)
      .chain(missing_added)
      .chain(context_added)
      .cloned()
      .collect();
    let removed_paths: ArcPathSet = compilation
      .removed_files
      .iter()
      .chain(file_removed)
      .chain(context_removed)
      .chain(missing_removed)
      .cloned()
      .collect();
    self.snapshot.remove(removed_paths.into_iter());
    self.snapshot.add(modified_paths.into_iter()).await;
    self
      .warnings
      .extend(self.build_deps.add(build_added.cloned()).await);

    self.save().await;

    let logger = compilation.get_logger("rspack.persistentCache");
    for msg in std::mem::take(&mut self.warnings) {
      logger.warn(msg);
    }
  }

  async fn before_build_module_graph(&mut self, make_artifact: &mut BuildModuleGraphArtifact) {
    // TODO When does not need to pass variables through make_artifact.state, use compilation.is_rebuild to check
    if matches!(
      make_artifact.state,
      BuildModuleGraphArtifactState::Uninitialized
    ) {
      match self.make_occasion.recovery().await {
        Ok(artifact) => *make_artifact = artifact,
        Err(err) => self.warnings.push(err.to_string()),
      }
    }
  }

  async fn after_build_module_graph(&mut self, make_artifact: &BuildModuleGraphArtifact) {
    self.make_occasion.save(make_artifact);
  }
}
