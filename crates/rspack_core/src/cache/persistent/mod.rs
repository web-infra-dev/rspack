pub mod build_dependencies;
pub mod codec;
pub mod context;
pub mod occasion;
pub mod snapshot;
pub mod storage;

use std::{
  hash::{DefaultHasher, Hash, Hasher},
  num::NonZeroUsize,
  sync::Arc,
};

use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, AsVec, Skip},
};
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_hash::{HashDigest, HashFunction, RspackHash};
use rspack_workspace::rspack_pkg_version;

use self::{
  build_dependencies::{BuildDeps, BuildDepsOptions},
  codec::CacheCodec,
  context::CacheContext,
  occasion::{MakeOccasion, MetaOccasion, MinimizeOccasion},
  snapshot::{Snapshot, SnapshotOptions},
  storage::{StorageOptions, VersionRetention, create_storage},
};
use super::Cache;
use crate::{Compilation, CompilationLogger, CompilationLogging, CompilerOptions, Logger};

const LOGGER_NAME: &str = "rspack.persistentCache";

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
  /// Guards `initialize` from running more than once per compiler instance
  initialized: bool,

  ctx: CacheContext,
  build_deps: BuildDeps,
  snapshot: Arc<Snapshot>,
  make_occasion: MakeOccasion,
  meta_occasion: MetaOccasion,
  minimize_occasion: MinimizeOccasion,
}

impl PersistentCache {
  pub fn new(
    compiler_path: &str,
    option: &PersistentCacheOptions,
    compiler_options: Arc<CompilerOptions>,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
    compilation_logging: CompilationLogging,
  ) -> Self {
    let project_root = if option.portable {
      Some(compiler_options.context.as_path().to_path_buf())
    } else {
      None
    };
    let codec = Arc::new(CacheCodec::new(project_root));
    let max_versions = match &option.storage {
      StorageOptions::FileSystem {
        max_versions: Some(max_versions),
        ..
      } => Some(*max_versions),
      _ => None,
    };
    let retention_scope = max_versions.map(|_| {
      retention_scope(
        compiler_options.context.as_str(),
        compiler_path,
        compiler_options.name.as_deref(),
      )
    });
    let retention = retention_scope
      .as_ref()
      .zip(max_versions)
      .map(|(scope, max_versions)| {
        VersionRetention::new(
          scope.clone(),
          NonZeroUsize::new(
            usize::try_from(max_versions.get()).expect("u32 fits in supported target usize"),
          )
          .expect("non-zero u32 remains non-zero as usize"),
        )
      });
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
      // Count retention is compiler-scoped, so its physical version directory
      // must use the same scope to prevent one compiler from deleting another's cache.
      if let Some(retention_scope) = &retention_scope {
        retention_scope.hash(&mut hasher);
      }
      hex::encode(hasher.finish().to_ne_bytes())
    };
    let storage = create_storage(
      option.storage.clone(),
      version,
      retention,
      intermediate_filesystem,
    );
    let snapshot = Arc::new(Snapshot::new(
      option.snapshot.clone(),
      input_filesystem.clone(),
      codec.clone(),
    ));

    Self {
      initialized: false,
      ctx: CacheContext::new(
        storage,
        option.readonly,
        CompilationLogger::new(LOGGER_NAME.to_string(), compilation_logging),
      ),
      build_deps: BuildDeps::new(
        &option.build_dependencies,
        input_filesystem,
        snapshot.clone(),
      ),
      snapshot,
      make_occasion: MakeOccasion::new(codec.clone()),
      meta_occasion: MetaOccasion::new(codec.clone()),
      minimize_occasion: MinimizeOccasion::new(codec),
    }
  }

  async fn initialize(&mut self) {
    if self.initialized {
      return;
    }
    self.initialized = true;

    // build_deps is the first validation step. If it fails or the build
    // dependencies have changed, only the BUILD scope is reset here; each
    // subsequent occasion resets itself when it is skipped or fails.
    self.ctx.load_build_deps(&mut self.build_deps).await;

    // meta: load or reset. make will handle itself in before_build_module_graph.
    self.ctx.load_occasion(&self.meta_occasion).await;
  }
}

fn retention_scope(context: &str, compiler_path: &str, compiler_name: Option<&str>) -> String {
  let mut hasher = RspackHash::new(&HashFunction::Xxhash64);
  write_length_framed(&mut hasher, context);
  write_length_framed(&mut hasher, compiler_path);
  if let Some(compiler_name) = compiler_name {
    hasher.write(&[1]);
    write_length_framed(&mut hasher, compiler_name);
  } else {
    hasher.write(&[0]);
  }
  hasher.digest(&HashDigest::Hex).encoded().into()
}

fn write_length_framed(hasher: &mut RspackHash, value: &str) {
  hasher.write(&(value.len() as u64).to_be_bytes());
  hasher.write(value.as_bytes());
}

#[async_trait::async_trait]
impl Cache for PersistentCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    self.ctx.logger().info("persistent cache enabled");
    self.initialize().await;

    if compilation.is_rebuild {
      return false;
    }
    // rebuild will pass modified_files and removed_files from js side,
    // so only calculate them when build.
    if let Some((is_hot_start, modified_paths, removed_paths)) =
      self.ctx.load_snapshot(&self.snapshot).await
    {
      compilation.modified_files.extend(modified_paths);
      compilation.removed_files.extend(removed_paths);
      return is_hot_start;
    }

    false
  }

  async fn after_compile(&mut self, compilation: &Compilation) {
    // save meta
    self.ctx.save_occasion(&self.meta_occasion, &());

    // save snapshot
    let (_, file_added, file_updated, file_removed) = compilation.file_dependencies();
    let (_, context_added, context_updated, context_removed) = compilation.context_dependencies();
    let (_, missing_added, missing_updated, missing_removed) = compilation.missing_dependencies();
    let (_, build_added, build_updated, _) = compilation.build_dependencies();
    self
      .ctx
      .save_snapshot(
        &self.snapshot,
        (
          file_added.chain(file_updated).cloned(),
          file_removed.cloned(),
        ),
        (
          context_added.chain(context_updated).cloned(),
          context_removed.cloned(),
        ),
        (
          missing_added.chain(missing_updated).cloned(),
          missing_removed.cloned(),
        ),
      )
      .await;
    self
      .ctx
      .save_build_deps(
        &mut self.build_deps,
        build_added.chain(build_updated).cloned(),
      )
      .await;

    self.ctx.save_storage();
    self.ctx.reset();
  }

  async fn before_build_module_graph(&mut self, compilation: &mut Compilation) {
    if compilation.is_rebuild {
      return;
    }

    if let Some(artifact) = self.ctx.load_occasion(&self.make_occasion).await {
      *compilation.build_module_graph_artifact = artifact;
      for (module, _) in compilation
        .build_module_graph_artifact
        .get_module_graph()
        .modules()
      {
        compilation.exports_info_artifact.new_exports_info(*module);
      }
    }
  }

  async fn after_build_module_graph(&mut self, compilation: &Compilation) {
    self.ctx.save_occasion(
      &self.make_occasion,
      &compilation.build_module_graph_artifact,
    );
  }

  async fn before_process_assets(&mut self, compilation: &mut Compilation) {
    if compilation.is_rebuild {
      return;
    }

    let artifact = self
      .ctx
      .load_occasion(&self.minimize_occasion)
      .await
      .unwrap_or_default();
    compilation.minimize_persistent_cache_artifact = Some(artifact);
  }

  async fn after_process_assets(&mut self, compilation: &Compilation) {
    if let Some(artifact) = &compilation.minimize_persistent_cache_artifact {
      self.ctx.save_occasion(&self.minimize_occasion, artifact);
    }
  }

  async fn close(&self) {
    self.ctx.flush_storage().await;
  }
}

#[cfg(test)]
mod tests {
  use super::retention_scope;

  #[test]
  fn retention_scope_frames_each_identity_component() {
    assert_ne!(
      retention_scope("ab", "c", Some("d")),
      retention_scope("a", "bc", Some("d"))
    );
    assert_ne!(
      retention_scope("context", "compiler", None),
      retention_scope("context", "compiler", Some(""))
    );
  }
}
