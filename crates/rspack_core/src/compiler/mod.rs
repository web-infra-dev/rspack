mod compilation;
mod hmr;
mod make;
mod queue;
mod resolver;

use std::collections::hash_map::{self, Entry};
use std::{path::Path, sync::Arc};

pub use compilation::*;
pub use make::MakeParam;
pub use queue::*;
pub use resolver::*;
use rspack_error::Result;
use rspack_fs::AsyncWritableFileSystem;
use rspack_futures::FuturesResults;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rustc_hash::FxHashMap as HashMap;
use swc_core::ecma::atoms::JsWord;
use tracing::instrument;

use crate::tree_shaking::visitor::SymbolRef;
use crate::{
  cache::Cache, fast_set, AssetEmittedArgs, CompilerOptions, Plugin, PluginDriver,
  SharedPluginDriver,
};
use crate::{ExportInfo, UsageState};

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
  pub fn new(
    options: CompilerOptions,
    plugins: Vec<Box<dyn Plugin>>,
    output_filesystem: T,
  ) -> Self {
    let options = Arc::new(options);

    let resolver_factory = Arc::new(ResolverFactory::new(options.resolve.clone()));
    let plugin_driver = Arc::new(PluginDriver::new(
      options.clone(),
      plugins,
      resolver_factory.clone(),
    ));
    let cache = Arc::new(Cache::new(options.clone()));

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        plugin_driver.clone(),
        resolver_factory.clone(),
        cache.clone(),
      ),
      output_filesystem,
      plugin_driver,
      resolver_factory,
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
    // TODO: clear the outdate cache entries in resolver,
    // TODO: maybe it's better to use external entries.
    self.plugin_driver.resolver_factory.clear_entries();

    fast_set(
      &mut self.compilation,
      Compilation::new(
        self.options.clone(),
        Default::default(),
        self.plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.cache.clone(),
      ),
    );

    self.plugin_driver.before_compile().await?;

    // Fake this compilation as *currently* rebuilding does not create a new compilation
    self
      .plugin_driver
      .this_compilation(&mut self.compilation)
      .await?;

    self
      .plugin_driver
      .compilation(&mut self.compilation)
      .await?;

    self
      .compile(MakeParam::ForceBuildDeps(Default::default()))
      .await?;
    self.cache.begin_idle();
    self.compile_done().await?;
    Ok(())
  }

  async fn compile(&mut self, params: MakeParam) -> Result<()> {
    let option = self.options.clone();
    self.compilation.make(params).await?;

    self
      .plugin_driver
      .finish_make(&mut self.compilation)
      .await?;

    self.compilation.finish(self.plugin_driver.clone()).await?;
    // by default include all module in final chunk
    self.compilation.include_module_ids = self
      .compilation
      .module_graph
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
      let mut exports_info_map: IdentifierMap<HashMap<JsWord, ExportInfo>> =
        IdentifierMap::default();
      self.compilation.used_symbol_ref.iter().for_each(|item| {
        let importer = item.importer();
        let name = match item {
          SymbolRef::Declaration(d) => d.exported(),
          SymbolRef::Indirect(_) => return,
          SymbolRef::Star(_) => return,
          SymbolRef::Usage(_, _, _) => return,
          SymbolRef::Url {
            importer,
            src,
            dep_id,
          } => return,
          SymbolRef::Worker {
            importer,
            src,
            dep_id,
          } => return,
        };
        match exports_info_map.entry(importer) {
          Entry::Occupied(mut occ) => {
            let export_info = ExportInfo::new(name.clone(), UsageState::Used);
            occ.get_mut().insert(name.clone(), export_info);
          }
          Entry::Vacant(vac) => {
            let mut map = HashMap::default();
            let export_info = ExportInfo::new(name.clone(), UsageState::Used);
            map.insert(name.clone(), export_info);
            vac.insert(map);
          }
        }
      });
      {
        // take the ownership to avoid rustc complain can't use `&` and `&mut` at the same time
        let mut mi_to_mgm = std::mem::take(
          &mut self
            .compilation
            .module_graph
            .module_identifier_to_module_graph_module,
        );
        for mgm in mi_to_mgm.values_mut() {
          // merge exports info
          if let Some(exports_map) = exports_info_map.remove(&mgm.module_identifier) {
            mgm.exports.exports.extend(exports_map.into_iter());
          }
        }
        self
          .compilation
          .module_graph
          .module_identifier_to_module_graph_module = mi_to_mgm;
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
    self.compilation.seal(self.plugin_driver.clone()).await?;

    self
      .plugin_driver
      .after_compile(&mut self.compilation)
      .await?;

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.take_diagnostic();
    self
      .compilation
      .push_batch_diagnostic(plugin_driver_diagnostics);

    Ok(())
  }

  #[instrument(name = "compile_done", skip_all)]
  async fn compile_done(&mut self) -> Result<()> {
    if !self.compilation.options.builtins.no_emit_assets {
      self.emit_assets().await?;
    }

    self.compilation.done(self.plugin_driver.clone()).await?;
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

    self.plugin_driver.emit(&mut self.compilation).await?;

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

    self.plugin_driver.after_emit(&mut self.compilation).await
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

      let asset_emitted_args = AssetEmittedArgs {
        filename,
        output_path,
        source: source.clone(),
        target_path: file_path.as_path(),
        compilation: &self.compilation,
      };
      self
        .plugin_driver
        .asset_emitted(&asset_emitted_args)
        .await?;
    }
    Ok(())
  }
}
