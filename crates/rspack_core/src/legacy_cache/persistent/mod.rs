pub mod build_dependencies;
pub mod context;
pub mod occasion;
mod seed;
pub mod snapshot;
pub mod storage;
pub mod validation;

use std::{
  borrow::Cow,
  hash::{DefaultHasher, Hash, Hasher},
  sync::Arc,
};

use cow_utils::CowUtils;
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_workspace::rspack_pkg_version;
use rustc_hash::FxHasher;

use self::{
  build_dependencies::BuildDeps,
  context::CacheContext,
  occasion::{MakeOccasion, MinimizeOccasion, SourceMapDevToolPluginOccasion},
  seed::CacheSeed,
  snapshot::Snapshot,
  storage::{CacheDirectory, create_storage},
  validation::CacheValidation,
};
use super::Cache;
use crate::{
  Compilation, CompilationLogger, CompilationLogging, CompilerOptions, Logger, ModuleRule,
  PersistentCacheOptions, PortableRoot, cache::CacheCodec,
};

const LOGGER_NAME: &str = "rspack.persistentCache";

/// The resolver configuration and module rules, with paths below the root spelled relative to it.
///
/// A resolution is reused only under an equal layout. Configured paths below the root are
/// computed for this checkout and move with it, so checkouts share their resolutions. A
/// configuration that names a path in another checkout renders differently, which expires the
/// resolutions and re-resolves them.
///
/// Rules render whole, in order, with their conditions and nesting: which rule matches a resource
/// decides its loaders and module type, and a fixed `include` or `exclude` stops matching when the
/// checkout moves although the configuration is unchanged. A rule with a JavaScript predicate
/// cannot be compared, so its layout also carries the root: the same checkout reuses, a moved
/// checkout re-resolves.
fn resolve_layout(options: &CompilerOptions, root: &PortableRoot) -> u64 {
  let rendered = format!(
    "{:?}{:?}{:?}",
    options.resolve, options.resolve_loader, options.module.rules
  );
  let mut rendered = Cow::Borrowed(rendered.as_str());
  for root in [&root.root, &root.canonical_root] {
    // Loader paths render raw, and other paths escaped as `Debug` does.
    let escaped = format!("{root:?}");
    for spelled in [root.as_str(), escaped.trim_matches('"')] {
      if let Cow::Owned(replaced) = rendered.cow_replace(spelled, "") {
        rendered = Cow::Owned(replaced);
      }
    }
  }
  let mut hasher = FxHasher::default();
  rendered.hash(&mut hasher);
  if options
    .module
    .rules
    .iter()
    .any(ModuleRule::has_opaque_condition)
  {
    // Spelled after root stripping, so the root itself stays in the layout.
    root.canonical_root.hash(&mut hasher);
  }
  hasher.finish()
}

/// Persistent cache implementation
#[derive(Debug)]
pub struct PersistentCache {
  /// Guards `initialize` from running more than once per compiler instance
  initialized: bool,
  seed: Option<CacheSeed>,
  warn_loader_requests: bool,

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
    let mut codec = CacheCodec::new(option.portable_root.clone());
    if let Some(root) = &option.portable_root {
      codec = codec.with_resolve_layout(resolve_layout(&compiler_options, root));
    }
    let codec = Arc::new(codec);
    // Each compiler path owns exactly one storage directory.
    let cache_directory = {
      let mut hasher = DefaultHasher::new();
      compiler_path.hash(&mut hasher);
      CacheDirectory::new(hex::encode(hasher.finish().to_ne_bytes()))
    };
    let seed = CacheSeed::new(option, &cache_directory, intermediate_filesystem.clone());
    let storage = create_storage(
      option.storage.clone(),
      cache_directory,
      option.max_age,
      intermediate_filesystem,
    );
    let snapshot = Arc::new(Snapshot::new(
      compiler_options.snapshot.clone(),
      compiler_options.context.clone(),
      input_filesystem.clone(),
      codec.clone(),
    ));

    Self {
      initialized: false,
      seed,
      warn_loader_requests: option.warn_loader_requests,
      ctx: CacheContext::new(
        storage,
        option.readonly,
        CompilationLogger::new(LOGGER_NAME.to_string(), compilation_logging),
      ),
      validation: CacheValidation::new(
        codec.clone(),
        format!(
          "{}|{}|{}",
          rspack_pkg_version!(),
          option.rebase_loader_requests,
          codec.validation_version(&option.version, compiler_options.context.as_path())
        ),
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

  /// Names the reason modules a loader bound to the root rebuild in another worktree, since nothing
  /// else shows it and only the project can decide that those paths move with the checkout.
  fn warn_absolute_loader_requests(&self, compilation: &Compilation) {
    let artifact = &compilation.build_module_graph_artifact;
    let module_graph = artifact.get_module_graph();
    let mut modules: Vec<_> = artifact
      .built_modules()
      .filter_map(|identifier| module_graph.module_by_identifier(identifier))
      .filter(|module| module.build_info().absolute_loader_requests)
      .map(|module| module.readable_identifier(&compilation.options.context))
      .collect();
    modules.sort_unstable();
    if let Some(first) = modules.first() {
      self.ctx.logger().warn(format!(
        "Modules with absolute requests written by loaders rebuild in another worktree: {} in this build, such as {first}. Set cache.portable.rebaseLoaderRequests to true to write those requests relative to the module. See https://rspack.rs/config/cache#portable",
        modules.len()
      ));
    }
  }

  async fn initialize(&mut self) {
    if self.initialized {
      return;
    }
    self.initialized = true;
    if let Some(seed) = self.seed.take() {
      seed.run(&self.validation, self.ctx.logger()).await;
    }
    self.ctx.cleanup_stale();

    self.ctx.validate(&mut self.validation).await;
  }
}

#[async_trait::async_trait]
impl Cache for PersistentCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    self.ctx.start_build();
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
    if self.warn_loader_requests && !compilation.is_rebuild {
      self.warn_absolute_loader_requests(compilation);
    }
    self.ctx.save_occasion(
      &self.make_occasion,
      &compilation.build_module_graph_artifact,
    );
  }

  async fn before_process_assets(&mut self, compilation: &mut Compilation) {
    if compilation.is_rebuild {
      return;
    }

    let cache_item = self
      .ctx
      .load_occasion(&self.minimize_occasion)
      .await
      .unwrap_or_default();
    compilation.minimize_persistent_cache = Some(cache_item);

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
