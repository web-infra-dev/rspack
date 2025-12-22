mod rebuild;
use std::sync::{Arc, atomic::AtomicU32};

use futures::future::join_all;
use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_fs::{IntermediateFileSystem, NativeFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_hook::define_hook;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_sources::BoxSource;
use rspack_tasks::{CompilerContext, within_compiler_context};
use rspack_util::{node_path::NodePath, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

pub use self::rebuild::CompilationRecords;
use crate::{
  BoxPlugin, CleanOptions, Compilation, CompilationAsset, CompilerOptions, ContextModuleFactory,
  Filename, KeepPattern, Logger, NormalModuleFactory, PluginDriver, ResolverFactory,
  SharedPluginDriver,
  cache::{Cache, new_cache},
  compilation::build_module_graph::ModuleExecutor,
  fast_set, include_hash,
  incremental::{Incremental, IncrementalPasses},
  old_cache::Cache as OldCache,
  trim_dir,
};

// should be SyncHook, but rspack need call js hook
define_hook!(CompilerThisCompilation: Series(compilation: &mut Compilation, params: &mut CompilationParams));
// should be SyncHook, but rspack need call js hook
define_hook!(CompilerCompilation: Series(compilation: &mut Compilation, params: &mut CompilationParams));
// should be AsyncParallelHook
define_hook!(CompilerMake: Series(compilation: &mut Compilation));
define_hook!(CompilerFinishMake: Series(compilation: &mut Compilation));
// should be SyncBailHook, but rspack need call js hook
define_hook!(CompilerShouldEmit: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilerEmit: Series(compilation: &mut Compilation));
define_hook!(CompilerAfterEmit: Series(compilation: &mut Compilation));
define_hook!(CompilerAssetEmitted: Series(compilation: &Compilation, filename: &str, info: &AssetEmittedInfo));
define_hook!(CompilerClose: Series(compilation: &Compilation));

#[derive(Debug, Default)]
pub struct CompilerHooks {
  pub this_compilation: CompilerThisCompilationHook,
  pub compilation: CompilerCompilationHook,
  pub make: CompilerMakeHook,
  pub finish_make: CompilerFinishMakeHook,
  pub should_emit: CompilerShouldEmitHook,
  pub emit: CompilerEmitHook,
  pub after_emit: CompilerAfterEmitHook,
  pub asset_emitted: CompilerAssetEmittedHook,
  pub close: CompilerCloseHook,
}

static COMPILER_ID: AtomicU32 = AtomicU32::new(0);

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CompilerId(u32);

impl CompilerId {
  pub fn new() -> Self {
    Self(COMPILER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
  }
}

impl Default for CompilerId {
  fn default() -> Self {
    Self::new()
  }
}
impl CompilerId {
  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

#[derive(Debug)]
pub struct Compiler {
  id: CompilerId,
  pub compiler_path: String,
  pub options: Arc<CompilerOptions>,
  pub output_filesystem: Arc<dyn WritableFileSystem>,
  pub intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
  pub input_filesystem: Arc<dyn ReadableFileSystem>,
  pub compilation: Compilation,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub cache: Box<dyn Cache>,
  pub old_cache: Arc<OldCache>,
  /// emitted asset versions
  /// the key of HashMap is filename, the value of HashMap is version
  pub emitted_asset_versions: HashMap<String, String>,
  compiler_context: Arc<CompilerContext>,
}

impl Compiler {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    compiler_path: String,
    options: CompilerOptions,
    plugins: Vec<BoxPlugin>,
    buildtime_plugins: Vec<BoxPlugin>,
    output_filesystem: Option<Arc<dyn WritableFileSystem>>,
    intermediate_filesystem: Option<Arc<dyn IntermediateFileSystem>>,
    // only supports passing input_filesystem in rust api, no support for js api
    input_filesystem: Option<Arc<dyn ReadableFileSystem>>,
    // no need to pass resolve_factory in rust api
    resolver_factory: Option<Arc<ResolverFactory>>,
    loader_resolver_factory: Option<Arc<ResolverFactory>>,
    compiler_context: Option<Arc<CompilerContext>>,
  ) -> Self {
    #[cfg(debug_assertions)]
    {
      if let Ok(mut debug_info) = crate::debug_info::DEBUG_INFO.lock() {
        debug_info.with_context(options.context.to_string());
      }
    }
    let pnp = options.resolve.pnp.unwrap_or(false);
    // pnp is only meaningful for input_filesystem, so disable it for intermediate_filesystem and output_filesystem
    let input_filesystem = input_filesystem.unwrap_or_else(|| Arc::new(NativeFileSystem::new(pnp)));

    let output_filesystem =
      output_filesystem.unwrap_or_else(|| Arc::new(NativeFileSystem::new(false)));
    let intermediate_filesystem =
      intermediate_filesystem.unwrap_or_else(|| Arc::new(NativeFileSystem::new(false)));

    let resolver_factory = resolver_factory.unwrap_or_else(|| {
      Arc::new(ResolverFactory::new(
        options.resolve.clone(),
        input_filesystem.clone(),
      ))
    });
    let loader_resolver_factory = loader_resolver_factory.unwrap_or_else(|| {
      Arc::new(ResolverFactory::new(
        options.resolve_loader.clone(),
        input_filesystem.clone(),
      ))
    });

    let options = Arc::new(options);
    let plugin_driver = PluginDriver::new(options.clone(), plugins, resolver_factory.clone());
    let buildtime_plugin_driver =
      PluginDriver::new(options.clone(), buildtime_plugins, resolver_factory.clone());
    let cache = new_cache(
      &compiler_path,
      options.clone(),
      input_filesystem.clone(),
      intermediate_filesystem.clone(),
    );
    let old_cache = Arc::new(OldCache::new(options.clone()));
    let incremental = Incremental::new_cold(options.experiments.incremental);
    let module_executor = ModuleExecutor::default();

    let id = CompilerId::new();
    let compiler_context = compiler_context.unwrap_or(Arc::new(CompilerContext::new()));
    Self {
      id,
      compiler_path,
      options: options.clone(),
      compilation: Compilation::new(
        id,
        options,
        plugin_driver.clone(),
        buildtime_plugin_driver.clone(),
        resolver_factory.clone(),
        loader_resolver_factory.clone(),
        None,
        old_cache.clone(),
        incremental,
        Some(module_executor),
        Default::default(),
        Default::default(),
        input_filesystem.clone(),
        intermediate_filesystem.clone(),
        output_filesystem.clone(),
        false,
        compiler_context.clone(),
      ),
      output_filesystem,
      intermediate_filesystem,
      plugin_driver,
      buildtime_plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      cache,
      old_cache,
      emitted_asset_versions: Default::default(),
      input_filesystem,
      compiler_context,
    }
  }

  pub fn id(&self) -> CompilerId {
    self.id
  }

  pub async fn run(&mut self) -> Result<()> {
    self.build().await?;
    Ok(())
  }
  pub async fn build(&mut self) -> Result<()> {
    let compiler_context = self.compiler_context.clone();
    within_compiler_context(compiler_context, self.build_inner()).await?;
    Ok(())
  }
  #[instrument("Compiler:build",target=TRACING_BENCH_TARGET, skip_all)]
  async fn build_inner(&mut self) -> Result<()> {
    self.old_cache.end_idle();
    // TODO: clear the outdated cache entries in resolver,
    // TODO: maybe it's better to use external entries.
    let plugin_driver_clone = self.plugin_driver.clone();
    let compilation_id = self.compilation.id();
    let _guard = scopeguard::guard((), move |_| plugin_driver_clone.clear_cache(compilation_id));

    fast_set(
      &mut self.compilation,
      Compilation::new(
        self.id,
        self.options.clone(),
        self.plugin_driver.clone(),
        self.buildtime_plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        None,
        self.old_cache.clone(),
        Incremental::new_cold(self.options.experiments.incremental),
        Some(Default::default()),
        Default::default(),
        Default::default(),
        self.input_filesystem.clone(),
        self.intermediate_filesystem.clone(),
        self.output_filesystem.clone(),
        false,
        self.compiler_context.clone(),
      ),
    );
    let _is_hot = self.cache.before_compile(&mut self.compilation).await;
    // TODO: disable it for now, enable it once persistent cache is added to all artifacts
    // if is_hot {
    //   // If it's a hot start, we can use incremental
    //   self.compilation.incremental = Incremental::new_hot(self.options.experiments.incremental);
    // }

    self.compile().await?;
    self.old_cache.begin_idle();
    self.compile_done().await?;
    self.cache.after_compile(&self.compilation).await;

    #[cfg(allocative)]
    crate::utils::snapshot_allocative("build");

    Ok(())
  }
  async fn build_module_graph(&mut self) -> Result<()> {
    let mut compilation_params = self.new_compilation_params();
    // FOR BINDING SAFETY:
    // Make sure `thisCompilation` hook was called for each `JsCompilation` update before any access to it.
    // `JsCompiler` tapped `thisCompilation` to update the `JsCompilation` on the JavaScript side.
    // Otherwise, trying to access the old native `JsCompilation` would cause undefined behavior
    // as the previous instance might get dropped.
    self
      .plugin_driver
      .compiler_hooks
      .this_compilation
      .call(&mut self.compilation, &mut compilation_params)
      .await?;
    self
      .plugin_driver
      .compiler_hooks
      .compilation
      .call(&mut self.compilation, &mut compilation_params)
      .await?;

    let logger = self.compilation.get_logger("rspack.Compiler");
    let make_start = logger.time("make");
    let make_hook_start = logger.time("make hook");
    self
      .cache
      .before_build_module_graph(&mut self.compilation.build_module_graph_artifact)
      .await;

    self
      .plugin_driver
      .compiler_hooks
      .make
      .call(&mut self.compilation)
      .await?;
    logger.time_end(make_hook_start);
    self.compilation.build_module_graph().await?;
    logger.time_end(make_start);

    let start = logger.time("finish make hook");
    self
      .plugin_driver
      .compiler_hooks
      .finish_make
      .call(&mut self.compilation)
      .await?;
    logger.time_end(start);

    let start = logger.time("finish compilation");
    self.compilation.finish_build_module_graph().await?;
    self
      .cache
      .after_build_module_graph(&self.compilation.build_module_graph_artifact)
      .await;

    logger.time_end(start);
    Ok(())
  }
  #[instrument("Compiler:compile", target=TRACING_BENCH_TARGET,skip_all)]
  async fn compile(&mut self) -> Result<()> {
    let logger = self.compilation.get_logger("rspack.Compiler");
    let start = logger.time("seal compilation");
    #[cfg(feature = "debug_tool")]
    {
      use rspack_util::debug_tool::wait_for_signal;
      wait_for_signal("seal compilation");
    }
    self.build_module_graph().await?;
    self
      .compilation
      .collect_build_module_graph_effects()
      .await?;
    self.compilation.seal(self.plugin_driver.clone()).await?;
    logger.time_end(start);

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.take_diagnostic();
    self
      .compilation
      .extend_diagnostics(plugin_driver_diagnostics);

    Ok(())
  }

  #[instrument("Compile:done", skip_all)]
  async fn compile_done(&mut self) -> Result<()> {
    let logger = self.compilation.get_logger("rspack.Compiler");

    if matches!(
      self
        .plugin_driver
        .compiler_hooks
        .should_emit
        .call(&mut self.compilation)
        .await?,
      Some(false)
    ) {
      return Ok(());
    }

    let start = logger.time("emitAssets");
    self.emit_assets().await?;
    logger.time_end(start);

    Ok(())
  }

  #[instrument("emit_assets", skip_all)]
  pub async fn emit_assets(&mut self) -> Result<()> {
    let output_path_str = self
      .compilation
      .get_path(
        &Filename::from(&self.options.output.path),
        Default::default(),
      )
      .await?;
    let output_path = Utf8Path::new(&output_path_str);
    self.run_clean_options(output_path).await?;

    self
      .plugin_driver
      .compiler_hooks
      .emit
      .call(&mut self.compilation)
      .await?;

    let mut new_emitted_asset_versions = HashMap::default();

    rspack_futures::scope(|token| {
      self
        .compilation
        .assets()
        .iter()
        .for_each(|(filename, asset)| {
          // collect version info to new_emitted_asset_versions
          if self
            .compilation
            .incremental
            .passes_enabled(IncrementalPasses::EMIT_ASSETS)
          {
            new_emitted_asset_versions.insert(filename.clone(), asset.info.version.clone());
          }

          if let Some(old_version) = self.emitted_asset_versions.get(filename)
            && old_version.as_str() == asset.info.version
            && !old_version.is_empty()
          {
            return;
          }

          // SAFETY: await immediately and trust caller to poll future entirely
          let s = unsafe { token.used((&self, filename, asset, output_path)) };

          s.spawn(|(this, filename, asset, output_path)| {
            this.emit_asset(output_path, filename, asset)
          });
        })
    })
    .await;

    self.emitted_asset_versions = new_emitted_asset_versions;

    self
      .plugin_driver
      .compiler_hooks
      .after_emit
      .call(&mut self.compilation)
      .await
  }
  async fn emit_asset(
    &self,
    output_path: &Utf8Path,
    filename: &str,
    asset: &CompilationAsset,
  ) -> Result<()> {
    if let Some(source) = asset.get_source() {
      let (target_file, query) = filename.split_once('?').unwrap_or((filename, ""));
      let file_path = output_path.node_join(target_file);
      self
        .output_filesystem
        .create_dir_all(
          file_path
            .parent()
            .unwrap_or_else(|| panic!("The parent of {file_path} can't found")),
        )
        .await?;

      let content = source.buffer();

      let mut immutable = asset.info.immutable.unwrap_or(false);
      if !query.is_empty() {
        immutable = immutable
          && (include_hash(target_file, &asset.info.content_hash)
            || include_hash(target_file, &asset.info.chunk_hash)
            || include_hash(target_file, &asset.info.full_hash));
      }

      let stat = self
        .output_filesystem
        .stat(file_path.as_path().as_ref())
        .await
        .ok();

      let need_write = if !self.options.output.compare_before_emit {
        // write when compare_before_emit is false
        true
      } else if !stat.as_ref().is_some_and(|stat| stat.is_file) {
        // write when not exists or not a file
        true
      } else if immutable {
        // do not write when asset is immutable and the file exists
        false
      } else if (content.len() as u64) == stat.as_ref().unwrap_or_else(|| unreachable!()).size {
        match self
          .output_filesystem
          .read_file(file_path.as_path().as_ref())
          .await
        {
          // write when content is different
          Ok(c) => content != c,
          // write when file can not be read
          Err(_) => true,
        }
      } else {
        // write if content length is different
        true
      };

      if need_write {
        self.output_filesystem.write(&file_path, &content).await?;
        self.compilation.emitted_assets.insert(filename.to_string());
      }

      let info = AssetEmittedInfo {
        output_path: output_path.to_owned(),
        source: source.clone(),
        target_path: file_path,
      };
      self
        .plugin_driver
        .compiler_hooks
        .asset_emitted
        .call(&self.compilation, filename, &info)
        .await?;
    }
    Ok(())
  }

  async fn run_clean_options(&mut self, output_path: &Utf8Path) -> Result<()> {
    let clean_options = &self.options.output.clean;

    // keep all
    if let CleanOptions::CleanAll(false) = clean_options {
      return Ok(());
    }

    if self.emitted_asset_versions.is_empty() {
      match clean_options {
        CleanOptions::CleanAll(true) => {
          self.output_filesystem.remove_dir_all(output_path).await?;
        }
        CleanOptions::KeepPath(p) => {
          let path = output_path.join(p);
          trim_dir(
            &*self.output_filesystem,
            output_path,
            KeepPattern::Path(&path),
          )
          .await?;
        }
        CleanOptions::KeepRegex(r) => {
          let keep_pattern = KeepPattern::Regex(r);
          trim_dir(&*self.output_filesystem, output_path, keep_pattern).await?;
        }
        CleanOptions::KeepFunc(f) => {
          let keep_pattern = KeepPattern::Func(f);
          trim_dir(&*self.output_filesystem, output_path, keep_pattern).await?;
        }
        _ => {}
      }

      return Ok(());
    }

    let assets = self.compilation.assets();
    join_all(
      self
        .emitted_asset_versions
        .iter()
        .filter_map(|(filename, _version)| {
          if !assets.contains_key(filename) {
            let filename = filename.to_owned();
            Some(async {
              if !clean_options.keep(&filename).await {
                let filename = output_path.join(filename);
                let _ = self.output_filesystem.remove_file(&filename).await;
              }
            })
          } else {
            None
          }
        }),
    )
    .await;

    Ok(())
  }

  pub fn new_compilation_params(&self) -> CompilationParams {
    CompilationParams {
      normal_module_factory: Arc::new(NormalModuleFactory::new(
        self.options.clone(),
        self.loader_resolver_factory.clone(),
        self.plugin_driver.clone(),
      )),
      context_module_factory: Arc::new(ContextModuleFactory::new(
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        self.plugin_driver.clone(),
      )),
    }
  }

  pub async fn close(&self) -> Result<()> {
    self
      .plugin_driver
      .compiler_hooks
      .close
      .call(&self.compilation)
      .await?;

    Ok(())
  }
}

#[derive(Debug)]
pub struct CompilationParams {
  pub normal_module_factory: Arc<NormalModuleFactory>,
  pub context_module_factory: Arc<ContextModuleFactory>,
}

#[derive(Debug)]
pub struct AssetEmittedInfo {
  pub source: BoxSource,
  pub output_path: Utf8PathBuf,
  pub target_path: Utf8PathBuf,
}
