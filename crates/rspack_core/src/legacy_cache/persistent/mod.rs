pub mod build_dependencies;
pub mod context;
pub mod occasion;
pub mod snapshot;
pub mod storage;
pub mod validation;

use std::{
  hash::{DefaultHasher, Hash, Hasher},
  sync::Arc,
};

use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_workspace::rspack_pkg_version;

use self::{
  build_dependencies::BuildDeps,
  context::CacheContext,
  occasion::{MakeOccasion, MinimizeOccasion, SourceMapDevToolPluginOccasion},
  snapshot::Snapshot,
  storage::{CacheDirectory, create_storage},
  validation::CacheValidation,
};
use super::Cache;
use crate::{
  Compilation, CompilationLogger, CompilationLogging, CompilerOptions, Logger,
  cache::{CacheCodec, PersistentCacheOptions},
};

const LOGGER_NAME: &str = "rspack.persistentCache";

/// Persistent cache implementation
#[derive(Debug)]
pub struct PersistentCache {
  /// Guards `initialize` from running more than once per compiler instance
  initialized: bool,

  ctx: CacheContext,
  validation: CacheValidation,
  snapshot: Arc<Snapshot>,
  make_occasion: MakeOccasion,
  minimize_occasion: MinimizeOccasion,
  source_map_dev_tool_plugin_occasion: SourceMapDevToolPluginOccasion,
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
    // Each compiler path owns exactly one storage directory.
    let cache_directory = {
      let mut hasher = DefaultHasher::new();
      compiler_path.hash(&mut hasher);
      CacheDirectory::new(hex::encode(hasher.finish().to_ne_bytes()))
    };
    let storage = create_storage(
      option.storage.clone(),
      cache_directory,
      option.max_age,
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
      validation: CacheValidation::new(
        codec.clone(),
        format!("{}|{}", rspack_pkg_version!(), option.version),
        BuildDeps::new(
          &option.build_dependencies,
          input_filesystem,
          snapshot.clone(),
        ),
      ),
      snapshot,
      make_occasion: MakeOccasion::new(codec.clone()),
      minimize_occasion: MinimizeOccasion::new(codec.clone()),
      source_map_dev_tool_plugin_occasion: SourceMapDevToolPluginOccasion::new(codec),
    }
  }

  async fn initialize(&mut self) {
    if self.initialized {
      return;
    }
    self.initialized = true;
    self.ctx.cleanup_stale();

    self.ctx.validate(&mut self.validation).await;
  }
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
    self.ctx.save_validation(&self.validation);

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
        &mut self.validation,
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

    if let Some(cache_item) = self.ctx.load_occasion(&self.make_occasion).await {
      *compilation.build_module_graph_artifact = cache_item;
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

    if !compilation.options.experiments.new_cache.minimize {
      let cache_item = self
        .ctx
        .load_occasion(&self.minimize_occasion)
        .await
        .unwrap_or_default();
      compilation.minimize_persistent_cache = Some(cache_item);
    }

    if compilation.use_source_map_dev_tool_plugin_cache {
      let cache_item = self
        .ctx
        .load_occasion(&self.source_map_dev_tool_plugin_occasion)
        .await
        .unwrap_or_default();
      compilation.source_map_dev_tool_plugin_cache = Some(cache_item);
    }
  }

  async fn after_process_assets(&mut self, compilation: &Compilation) {
    if let Some(cache_item) = &compilation.minimize_persistent_cache {
      self.ctx.save_occasion(&self.minimize_occasion, cache_item);
    }
    if compilation.use_source_map_dev_tool_plugin_cache
      && let Some(cache_item) = &compilation.source_map_dev_tool_plugin_cache
    {
      self
        .ctx
        .save_occasion(&self.source_map_dev_tool_plugin_occasion, cache_item);
    }
  }

  async fn close(&self) {
    self.ctx.flush_storage().await;
  }
}
