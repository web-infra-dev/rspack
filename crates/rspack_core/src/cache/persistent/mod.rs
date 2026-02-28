pub mod build_dependencies;
pub mod codec;
pub mod occasion;
pub mod snapshot;
pub mod storage;

use std::{
  hash::{DefaultHasher, Hash, Hasher},
  sync::Arc,
};

use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, AsVec, Skip},
};
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_paths::ArcPathSet;
use rspack_workspace::rspack_pkg_version;

use self::{
  build_dependencies::{BuildDeps, BuildDepsOptions},
  codec::CacheCodec,
  occasion::{MakeOccasion, MetaOccasion},
  snapshot::{Snapshot, SnapshotOptions, SnapshotScope},
  storage::{Storage, StorageOptions, create_storage},
};
use super::Cache;
use crate::{BuildModuleGraphArtifactState, Compilation, CompilerOptions, Logger};

#[cacheable]
#[derive(Debug, Clone, Hash)]
pub struct PersistentCacheOptions {
  #[cacheable(with=AsVec<As<PortablePath>>)]
  pub build_dependencies: BuildDepsOptions,
  pub version: String,
  pub snapshot: SnapshotOptions,
  pub storage: StorageOptions,
  pub portable: bool,
  #[cacheable(with=Skip)]
  pub readonly: bool,
}

/// Persistent cache implementation
#[derive(Debug)]
pub struct PersistentCache {
  initialized: bool,
  valid: bool,
  readonly: bool,
  build_deps: BuildDeps,
  snapshot: Arc<Snapshot>,
  make_occasion: MakeOccasion,
  meta_occasion: MetaOccasion,
  async_mode: bool,
  storage: Arc<dyn Storage>,
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
    let project_root = if option.portable {
      Some(compiler_options.context.as_path().to_path_buf())
    } else {
      None
    };
    let codec = Arc::new(CacheCodec::new(project_root));
    // use codec.encode to transform the absolute path in option,
    // it will ensure that same project in different directory have the same version.
    let option_bytes = codec
      .encode(option)
      .expect("should persistent cache options can be serialized");
    let version = {
      let mut hasher = DefaultHasher::new();
      compiler_path.hash(&mut hasher);
      option_bytes.hash(&mut hasher);
      rspack_pkg_version!().hash(&mut hasher);
      compiler_options.name.hash(&mut hasher);
      compiler_options.mode.hash(&mut hasher);
      hex::encode(hasher.finish().to_ne_bytes())
    };
    let storage = create_storage(option.storage.clone(), version, intermediate_filesystem);
    let snapshot = Arc::new(Snapshot::new(
      option.snapshot.clone(),
      input_filesystem.clone(),
      storage.clone(),
      codec.clone(),
    ));

    Self {
      initialized: false,
      valid: false,
      readonly: option.readonly,
      build_deps: BuildDeps::new(
        &option.build_dependencies,
        input_filesystem,
        snapshot.clone(),
      ),
      snapshot,
      make_occasion: MakeOccasion::new(storage.clone(), codec.clone()),
      meta_occasion: MetaOccasion::new(storage.clone(), codec),
      warnings: Default::default(),
      async_mode,
      storage,
    }
  }

  async fn initialize(&mut self) {
    if self.initialized {
      return;
    }
    self.initialized = true;

    match self.build_deps.validate().await {
      Ok(success) => {
        self.valid = success;
      }
      Err(err) => {
        self.valid = false;
        self.warnings.push(err.to_string());
      }
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
    if self.valid && !compilation.is_rebuild {
      let mut is_hot_start = false;
      let mut modified_paths = ArcPathSet::default();
      let mut removed_paths = ArcPathSet::default();
      let data = vec![
        self.snapshot.calc_modified_paths(SnapshotScope::FILE).await,
        self
          .snapshot
          .calc_modified_paths(SnapshotScope::CONTEXT)
          .await,
        self
          .snapshot
          .calc_modified_paths(SnapshotScope::MISSING)
          .await,
      ];
      for item in data {
        match item {
          Ok((a, b, c, _)) => {
            is_hot_start = is_hot_start || a;
            modified_paths.extend(b);
            removed_paths.extend(c);
          }
          Err(err) => {
            self.warnings.push(err.to_string());
            return false;
          }
        };
      }

      tracing::debug!("cache::snapshot recovery {modified_paths:?} {removed_paths:?}",);
      compilation.modified_files.extend(modified_paths);
      compilation.removed_files.extend(removed_paths);
      return is_hot_start;
    }
    false
  }

  async fn after_compile(&mut self, compilation: &Compilation) {
    // skip storage reset and save if cache is readonly
    if !self.readonly {
      if !self.valid {
        // reset before save write data
        self.storage.reset().await;
        self.valid = true;
      }
      // save meta
      self.meta_occasion.save();

      // save snapshot
      // TODO add a all_dependencies to collect dependencies
      let (_, file_added, file_updated, file_removed) = compilation.file_dependencies();
      let (_, context_added, context_updated, context_removed) = compilation.context_dependencies();
      let (_, missing_added, missing_updated, missing_removed) = compilation.missing_dependencies();
      let (_, build_added, build_updated, _) = compilation.build_dependencies();
      self
        .snapshot
        .remove(SnapshotScope::FILE, file_removed.cloned());
      self
        .snapshot
        .remove(SnapshotScope::CONTEXT, context_removed.cloned());
      self
        .snapshot
        .remove(SnapshotScope::MISSING, missing_removed.cloned());
      self
        .snapshot
        .add(SnapshotScope::FILE, file_added.chain(file_updated).cloned())
        .await;
      self
        .snapshot
        .add(
          SnapshotScope::CONTEXT,
          context_added.chain(context_updated).cloned(),
        )
        .await;
      self
        .snapshot
        .add(
          SnapshotScope::MISSING,
          missing_added.chain(missing_updated).cloned(),
        )
        .await;
      self.warnings.extend(
        self
          .build_deps
          .add(build_added.chain(build_updated).cloned())
          .await,
      );

      self.save().await;
    }

    let logger = compilation.get_logger("rspack.persistentCache");
    for msg in std::mem::take(&mut self.warnings) {
      logger.warn(msg);
    }
  }

  async fn before_build_module_graph(&mut self, compilation: &mut Compilation) {
    // TODO When does not need to pass variables through make_artifact.state, use compilation.is_rebuild to check
    if self.valid
      && matches!(
        compilation.build_module_graph_artifact.state,
        BuildModuleGraphArtifactState::Uninitialized
      )
    {
      match self.make_occasion.recovery().await {
        Ok(artifact) => {
          *compilation.build_module_graph_artifact = artifact;
          for (module, _) in compilation
            .build_module_graph_artifact
            .get_module_graph()
            .modules()
          {
            compilation.exports_info_artifact.new_exports_info(*module);
          }
        }
        Err(err) => self.warnings.push(err.to_string()),
      }
    }
  }

  async fn after_build_module_graph(&self, compilation: &Compilation) {
    if !self.readonly {
      self
        .make_occasion
        .save(&compilation.build_module_graph_artifact);
    }
  }
}
