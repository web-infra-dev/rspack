mod compilation;
mod execute_module;
mod hmr;
mod make;

use std::collections::hash_map::Entry;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::AsyncWritableFileSystem;
use rspack_futures::FuturesResults;
use rspack_hook::{AsyncParallel3Hook, AsyncSeries2Hook, AsyncSeriesBailHook, AsyncSeriesHook};
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rustc_hash::FxHashMap as HashMap;
use swc_core::ecma::atoms::Atom;
use tracing::instrument;

pub use self::compilation::*;
pub use self::execute_module::ExecuteModuleId;
pub use self::hmr::{collect_changed_modules, CompilationRecords};
pub use self::make::{
  AddQueueHandler, BuildQueueHandler, BuildTimeExecutionOption, BuildTimeExecutionQueueHandler,
  BuildTimeExecutionTask, FactorizeQueueHandler, FactorizeTask, MakeParam,
  ProcessDependenciesQueueHandler,
};
use crate::cache::Cache;
use crate::tree_shaking::symbol::{IndirectType, StarSymbolKind, DEFAULT_JS_WORD};
use crate::tree_shaking::visitor::SymbolRef;
use crate::{
  fast_set, AssetEmittedInfo, CompilerOptions, Logger, ModuleGraph, PluginDriver, ResolverFactory,
  SharedPluginDriver,
};
use crate::{BoxPlugin, ExportInfo, UsageState};
use crate::{CompilationParams, ContextModuleFactory, NormalModuleFactory};

// should be SyncHook, but rspack need call js hook
pub type CompilerThisCompilationHook = AsyncSeries2Hook<Compilation, CompilationParams>;
// should be SyncHook, but rspack need call js hook
pub type CompilerCompilationHook = AsyncSeries2Hook<Compilation, CompilationParams>;
// should be AsyncParallelHook, but rspack need add MakeParam to incremental rebuild
pub type CompilerMakeHook = AsyncSeries2Hook<Compilation, Vec<MakeParam>>;
pub type CompilerFinishMakeHook = AsyncSeriesHook<Compilation>;
// should be SyncBailHook, but rspack need call js hook
pub type CompilerShouldEmitHook = AsyncSeriesBailHook<Compilation, bool>;
pub type CompilerEmitHook = AsyncSeriesHook<Compilation>;
pub type CompilerAfterEmitHook = AsyncSeriesHook<Compilation>;
// should be AsyncSeriesHook, but rspack parallel emit asset, only accept immutable params,
// and it has no effect about mutate the params in webpack
pub type CompilerAssetEmittedHook = AsyncParallel3Hook<Compilation, String, AssetEmittedInfo>;

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
pub struct Compiler<T>
where
  T: AsyncWritableFileSystem + Send + Sync,
{
  pub options: Arc<CompilerOptions>,
  pub output_filesystem: T,
  pub compilation: Compilation,
  pub plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub cache: Arc<Cache>,
  /// emitted asset versions
  /// the key of HashMap is filename, the value of HashMap is version
  pub emitted_asset_versions: HashMap<String, String>,
}

impl<T> Compiler<T>
where
  T: AsyncWritableFileSystem + Send + Sync,
{
  #[instrument(skip_all)]
  pub fn new(options: CompilerOptions, plugins: Vec<BoxPlugin>, output_filesystem: T) -> Self {
    #[cfg(debug_assertions)]
    {
      if let Ok(mut debug_info) = crate::debug_info::DEBUG_INFO.lock() {
        debug_info.with_context(options.context.to_string());
      }
    }
    let resolver_factory = Arc::new(ResolverFactory::new(options.resolve.clone()));
    let loader_resolver_factory = Arc::new(ResolverFactory::new(options.resolve_loader.clone()));
    let (plugin_driver, options) = PluginDriver::new(options, plugins, resolver_factory.clone());
    let cache = Arc::new(Cache::new(options.clone()));
    assert!(!(options.is_new_tree_shaking() && options.builtins.tree_shaking.enable()), "Can't enable builtins.tree_shaking and `experiments.rspack_future.new_treeshaking` at the same time");
    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        ModuleGraph::default(),
        plugin_driver.clone(),
        resolver_factory.clone(),
        loader_resolver_factory.clone(),
        None,
        cache.clone(),
      ),
      output_filesystem,
      plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      cache,
      emitted_asset_versions: Default::default(),
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    self.build().await?;
    Ok(())
  }

  #[instrument(name = "build", skip_all)]
  pub async fn build(&mut self) -> Result<()> {
    self.cache.end_idle();
    // TODO: clear the outdated cache entries in resolver,
    // TODO: maybe it's better to use external entries.
    self.plugin_driver.resolver_factory.clear_cache();

    fast_set(
      &mut self.compilation,
      Compilation::new(
        self.options.clone(),
        ModuleGraph::default(),
        self.plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.loader_resolver_factory.clone(),
        None,
        self.cache.clone(),
      ),
    );

    self
      .compile(vec![MakeParam::ForceBuildDeps(Default::default())])
      .await?;
    self.cache.begin_idle();
    self.compile_done().await?;
    Ok(())
  }

  #[instrument(name = "compile", skip_all)]
  async fn compile(&mut self, mut params: Vec<MakeParam>) -> Result<()> {
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
    let option = self.options.clone();
    let make_start = logger.time("make");
    let make_hook_start = logger.time("make hook");
    if let Some(e) = self
      .plugin_driver
      .compiler_hooks
      .make
      .call(&mut self.compilation, &mut params)
      .await
      .err()
    {
      self.compilation.push_batch_diagnostic(vec![e.into()]);
    }
    logger.time_end(make_hook_start);
    self.compilation.make(params).await?;
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
    // by default include all module in final chunk
    self.compilation.include_module_ids = self
      .compilation
      .get_module_graph()
      .modules()
      .keys()
      .cloned()
      .collect::<IdentifierSet>();

    if option.builtins.tree_shaking.enable()
      || option
        .output
        .enabled_library_types
        .as_ref()
        .map(|types| {
          types
            .iter()
            .any(|item| item == "module" || item == "commonjs-static")
        })
        .unwrap_or(false)
    {
      let (analyze_result, diagnostics) = self
        .compilation
        .optimize_dependency()
        .await?
        .split_into_parts();
      if !diagnostics.is_empty() {
        self.compilation.push_batch_diagnostic(diagnostics);
      }
      self.compilation.used_symbol_ref = analyze_result.used_symbol_ref;
      let mut exports_info_map: IdentifierMap<HashMap<Atom, ExportInfo>> = IdentifierMap::default();
      self.compilation.used_symbol_ref.iter().for_each(|item| {
        let (importer, name) = match item {
          SymbolRef::Declaration(d) => (d.src(), d.exported()),
          SymbolRef::Indirect(i) => match i.ty {
            IndirectType::Import(_, _) => (i.src(), i.indirect_id()),
            IndirectType::ImportDefault(_) => (i.src(), DEFAULT_JS_WORD.deref()),
            IndirectType::ReExport(_, _) => (i.importer(), i.id()),
            _ => return,
          },
          SymbolRef::Star(s) => match s.ty() {
            StarSymbolKind::ReExportAllAs => (s.module_ident(), s.binding()),
            _ => return,
          },
          SymbolRef::Usage(_, _, _) => return,
          SymbolRef::Url { .. } => return,
          SymbolRef::Worker { .. } => return,
        };
        match exports_info_map.entry(importer) {
          Entry::Occupied(mut occ) => {
            let export_info = ExportInfo::new(Some(name.clone()), UsageState::Used, None);
            occ.get_mut().insert(name.clone(), export_info);
          }
          Entry::Vacant(vac) => {
            let mut map = HashMap::default();
            let export_info = ExportInfo::new(Some(name.clone()), UsageState::Used, None);
            map.insert(name.clone(), export_info);
            vac.insert(map);
          }
        }
      });
      {
        let module_graph = self.compilation.get_module_graph_mut();
        for (module_identifier, exports_map) in exports_info_map.into_iter() {
          let exports_id = module_graph
            .module_graph_module_by_identifier(&module_identifier)
            .map(|mgm| mgm.exports);
          if let Some(exports_id) = &exports_id {
            for (name, export_info) in exports_map {
              let exports = module_graph.get_exports_info_mut_by_id(exports_id);
              let export_id = export_info.id;
              exports.exports.insert(name, export_id);
              module_graph.set_export_info(export_id, export_info);
            }
          }
        }
      }

      self.compilation.bailout_module_identifiers = analyze_result.bail_out_module_identifiers;
      self.compilation.side_effects_free_modules = analyze_result.side_effects_free_modules;
      self.compilation.module_item_map = analyze_result.module_item_map;
      if self.options.builtins.tree_shaking.enable()
        && self.options.optimization.side_effects.is_enable()
      {
        self.compilation.include_module_ids = analyze_result.include_module_ids;
      }
      self.compilation.optimize_analyze_result_map = analyze_result.analyze_results;
    }
    let start = logger.time("seal compilation");
    self.compilation.seal(self.plugin_driver.clone()).await?;
    logger.time_end(start);

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.take_diagnostic();
    self
      .compilation
      .push_batch_diagnostic(plugin_driver_diagnostics);

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
      return self.compilation.done(self.plugin_driver.clone()).await;
    }

    let start = logger.time("emitAssets");
    self.emit_assets().await?;
    logger.time_end(start);

    let start = logger.time("done hook");
    self.compilation.done(self.plugin_driver.clone()).await?;
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
              let file_path = Path::new(&self.options.output.path).join(filename);
              Some(self.output_filesystem.remove_file(file_path))
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
        if self.options.is_incremental_rebuild_emit_asset_enabled() {
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

  async fn emit_asset(
    &self,
    output_path: &Path,
    filename: &str,
    asset: &CompilationAsset,
  ) -> Result<()> {
    if let Some(source) = asset.get_source() {
      let filename = filename
        .split_once('?')
        .map(|(filename, _query)| filename)
        .unwrap_or(filename);
      let file_path = Path::new(&output_path).join(filename);
      self
        .output_filesystem
        .create_dir_all(
          file_path
            .parent()
            .unwrap_or_else(|| panic!("The parent of {} can't found", file_path.display())),
        )
        .await?;
      self
        .output_filesystem
        .write(&file_path, source.buffer())
        .await?;

      self.compilation.emitted_assets.insert(filename.to_string());

      let info = AssetEmittedInfo {
        output_path: output_path.to_owned(),
        source: source.clone(),
        target_path: file_path,
      };
      self
        .plugin_driver
        .compiler_hooks
        .asset_emitted
        .call(&self.compilation, &filename.to_owned(), &info)
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
        self.cache.clone(),
      )),
      context_module_factory: Arc::new(ContextModuleFactory::new(
        self.loader_resolver_factory.clone(),
        self.plugin_driver.clone(),
        self.cache.clone(),
      )),
    }
  }
}
