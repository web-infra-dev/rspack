mod compilation;
mod hmr;
mod make;
mod module_executor;
use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::{AsyncWritableFileSystem, NativeFileSystem, ReadableFileSystem};
use rspack_futures::FuturesResults;
use rspack_hook::define_hook;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_sources::BoxSource;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

pub use self::compilation::*;
pub use self::hmr::{collect_changed_modules, CompilationRecords};
pub use self::module_executor::{ExecuteModuleId, ExecutedRuntimeModule, ModuleExecutor};
use crate::cache::{new_cache, Cache};
use crate::incremental::IncrementalPasses;
use crate::old_cache::Cache as OldCache;
use crate::{
  fast_set, include_hash, BoxPlugin, CompilerOptions, Logger, PluginDriver, ResolverFactory,
  SharedPluginDriver,
};
use crate::{ContextModuleFactory, NormalModuleFactory};

// should be SyncHook, but rspack need call js hook
define_hook!(CompilerThisCompilation: AsyncSeries(compilation: &mut Compilation, params: &mut CompilationParams));
// should be SyncHook, but rspack need call js hook
define_hook!(CompilerCompilation: AsyncSeries(compilation: &mut Compilation, params: &mut CompilationParams));
// should be AsyncParallelHook
define_hook!(CompilerMake: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilerFinishMake: AsyncSeries(compilation: &mut Compilation));
// should be SyncBailHook, but rspack need call js hook
define_hook!(CompilerShouldEmit: AsyncSeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilerEmit: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilerAfterEmit: AsyncSeries(compilation: &mut Compilation));
define_hook!(CompilerAssetEmitted: AsyncSeries(compilation: &Compilation, filename: &str, info: &AssetEmittedInfo));

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
}

#[derive(Debug)]
pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub output_filesystem: Box<dyn AsyncWritableFileSystem + Send + Sync>,
  pub input_filesystem: Arc<dyn ReadableFileSystem>,
  pub compilation: Compilation,
  pub plugin_driver: SharedPluginDriver,
  pub buildtime_plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub cache: Arc<dyn Cache>,
  pub old_cache: Arc<OldCache>,
  /// emitted asset versions
  /// the key of HashMap is filename, the value of HashMap is version
  pub emitted_asset_versions: HashMap<String, String>,
}

impl Compiler {
  #[instrument(skip_all)]
  pub fn new(
    options: CompilerOptions,
    plugins: Vec<BoxPlugin>,
    buildtime_plugins: Vec<BoxPlugin>,
    output_filesystem: Option<Box<dyn AsyncWritableFileSystem + Send + Sync>>,
    // only supports passing input_filesystem in rust api, no support for js api
    input_filesystem: Option<Arc<dyn ReadableFileSystem + Send + Sync>>,
    // no need to pass resolve_factory in rust api
    resolver_factory: Option<Arc<ResolverFactory>>,
    loader_resolver_factory: Option<Arc<ResolverFactory>>,
  ) -> Self {
    #[cfg(debug_assertions)]
    {
      if let Ok(mut debug_info) = crate::debug_info::DEBUG_INFO.lock() {
        debug_info.with_context(options.context.to_string());
      }
    }
    let input_filesystem = input_filesystem.unwrap_or_else(|| Arc::new(NativeFileSystem {}));

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
    let cache = new_cache(options.clone(), input_filesystem.clone());
    let old_cache = Arc::new(OldCache::new(options.clone()));
    let module_executor = ModuleExecutor::default();
    let output_filesystem = output_filesystem.unwrap_or_else(|| Box::new(NativeFileSystem {}));

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        plugin_driver.clone(),
        buildtime_plugin_driver.clone(),
        resolver_factory.clone(),
        loader_resolver_factory.clone(),
        None,
        cache.clone(),
        old_cache.clone(),
        Some(module_executor),
        Default::default(),
        Default::default(),
        input_filesystem.clone(),
      ),
      output_filesystem,
      plugin_driver,
      buildtime_plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      cache,
      old_cache,
      emitted_asset_versions: Default::default(),
      input_filesystem,
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    self.build().await?;
    Ok(())
  }

  #[instrument(name = "build", skip_all)]
  pub async fn build(&mut self) -> Result<()> {
    self.old_cache.end_idle();
    // TODO: clear the outdated cache entries in resolver,
    // TODO: maybe it's better to use external entries.
    self.plugin_driver.clear_cache();

    fast_set(
      &mut self.compilation,
      Compilation::new(
        self.options.clone(),
        self.plugin_driver.clone(),
        self.buildtime_plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        None,
        self.cache.clone(),
        self.old_cache.clone(),
        Some(Default::default()),
        Default::default(),
        Default::default(),
        self.input_filesystem.clone(),
      ),
    );
    self.cache.before_compile(&mut self.compilation);

    self.compile().await?;
    self.old_cache.begin_idle();
    self.compile_done().await?;
    self.cache.after_compile(&self.compilation);
    Ok(())
  }

  #[instrument(name = "compile", skip_all)]
  async fn compile(&mut self) -> Result<()> {
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
    if let Some(e) = self
      .plugin_driver
      .compiler_hooks
      .make
      .call(&mut self.compilation)
      .await
      .err()
    {
      self.compilation.extend_diagnostics(vec![e.into()]);
    }
    logger.time_end(make_hook_start);
    self.compilation.make().await?;
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
    self.compilation.finish(self.plugin_driver.clone()).await?;
    logger.time_end(start);
    let start = logger.time("seal compilation");
    self.compilation.seal(self.plugin_driver.clone()).await?;
    logger.time_end(start);

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.take_diagnostic();
    self
      .compilation
      .extend_diagnostics(plugin_driver_diagnostics);

    Ok(())
  }

  #[instrument(name = "compile_done", skip_all)]
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

  #[instrument(name = "emit_assets", skip_all)]
  pub async fn emit_assets(&mut self) -> Result<()> {
    if self.options.output.clean {
      if self.emitted_asset_versions.is_empty() {
        self
          .output_filesystem
          .remove_dir_all(&self.options.output.path)
          .await?;
      } else {
        // clean unused file
        let assets = self.compilation.assets();
        let _ = self
          .emitted_asset_versions
          .iter()
          .filter_map(|(filename, _version)| {
            if !assets.contains_key(filename) {
              let filename = filename.to_owned();
              Some(async {
                let filename = Utf8Path::new(&self.options.output.path).join(filename);
                let _ = self.output_filesystem.remove_file(&filename).await;
              })
            } else {
              None
            }
          })
          .collect::<FuturesResults<_>>();
      }
    }

    self
      .plugin_driver
      .compiler_hooks
      .emit
      .call(&mut self.compilation)
      .await?;

    let mut new_emitted_asset_versions = HashMap::default();
    let results = self
      .compilation
      .assets()
      .iter()
      .filter_map(|(filename, asset)| {
        // collect version info to new_emitted_asset_versions
        if self
          .options
          .experiments
          .incremental
          .contains(IncrementalPasses::EMIT_ASSETS)
        {
          new_emitted_asset_versions.insert(filename.to_string(), asset.info.version.clone());
        }

        if let Some(old_version) = self.emitted_asset_versions.get(filename) {
          if old_version.as_str() == asset.info.version && !old_version.is_empty() {
            return None;
          }
        }

        Some(self.emit_asset(&self.options.output.path, filename, asset))
      })
      .collect::<FuturesResults<_>>();

    self.emitted_asset_versions = new_emitted_asset_versions;
    // return first error
    for item in results.into_inner() {
      item?;
    }

    self
      .plugin_driver
      .compiler_hooks
      .after_emit
      .call(&mut self.compilation)
      .await
  }

  #[instrument(skip_all, fields(filename = filename))]
  async fn emit_asset(
    &self,
    output_path: &Utf8Path,
    filename: &str,
    asset: &CompilationAsset,
  ) -> Result<()> {
    if let Some(source) = asset.get_source() {
      let (filename, query) = filename.split_once('?').unwrap_or((filename, ""));
      let file_path = output_path.join(filename);
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
          && (include_hash(filename, &asset.info.content_hash)
            || include_hash(filename, &asset.info.chunk_hash)
            || include_hash(filename, &asset.info.full_hash));
      }

      let stat = match self
        .output_filesystem
        .stat(file_path.as_path().as_ref())
        .await
      {
        Ok(stat) => Some(stat),
        Err(_) => None,
      };

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

  fn new_compilation_params(&self) -> CompilationParams {
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
