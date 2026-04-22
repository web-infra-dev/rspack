pub mod build_dependencies;
pub mod codec;
pub mod context;
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
use rspack_workspace::rspack_pkg_version;

use self::{
  build_dependencies::{BuildDeps, BuildDepsOptions},
  codec::CacheCodec,
  context::CacheContext,
  occasion::{MakeOccasion, MetaOccasion, MinimizeOccasion},
  snapshot::{Snapshot, SnapshotOptions},
  storage::{StorageOptions, create_storage},
};
use super::Cache;
use crate::{
  Compilation, CompilerOptions, GeneratorOptions, Logger, Module, ModuleIdentifier, ModuleType,
  ParserOptions,
};

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
  ) -> Self {
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
      codec.clone(),
    ));

    Self {
      initialized: false,
      ctx: CacheContext::new(storage, option.readonly),
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

async fn restore_parser_and_generators(compilation: &mut Compilation) {
  let normal_modules = {
    let module_graph = compilation.get_module_graph();
    module_graph
      .modules()
      .filter_map(|(module_identifier, module)| {
        module.as_normal_module().map(|normal_module| {
          (
            *module_identifier,
            *normal_module.module_type(),
            normal_module.get_parser_options().cloned(),
            normal_module.get_generator_options().cloned(),
          )
        })
      })
      .collect::<Vec<(
        ModuleIdentifier,
        ModuleType,
        Option<ParserOptions>,
        Option<GeneratorOptions>,
      )>>()
  };

  let Some(normal_module_factory) = &compilation.normal_module_factory else {
    return;
  };
  for (module_identifier, module_type, parser_options, generator_options) in normal_modules {
    let parser_and_generator = normal_module_factory
      .create_parser_and_generator(
        &module_type,
        parser_options.as_ref(),
        generator_options.as_ref(),
      )
      .await
      .expect("should restore parser_and_generator for cached NormalModule");
    normal_module_factory.set_parser_and_generator(module_identifier, parser_and_generator.into());
  }
}

#[async_trait::async_trait]
impl Cache for PersistentCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    self.initialize().await;

    if compilation.is_rebuild {
      return false;
    }
    // rebuild will pass modified_files and removed_files from js side,
    // so only calculate them when build.
    if let Some((is_hot_start, modified_paths, removed_paths)) =
      self.ctx.load_snapshot(&self.snapshot).await
    {
      tracing::debug!("cache::snapshot recovery {modified_paths:?} {removed_paths:?}",);
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

    let logger = compilation.get_logger("rspack.persistentCache");
    for msg in self.ctx.reset() {
      logger.warn(msg);
    }
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
      restore_parser_and_generators(compilation).await;
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
