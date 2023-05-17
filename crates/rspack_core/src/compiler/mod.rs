mod compilation;
mod hmr;
mod queue;
mod resolver;

use std::{path::Path, sync::Arc};

pub use compilation::*;
pub use queue::*;
pub use resolver::*;
use rspack_error::Result;
use rspack_fs::AsyncWritableFileSystem;
use rspack_futures::FuturesResults;
use rspack_identifier::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
  cache::Cache, fast_set, tree_shaking::webpack_ext::ExportInfoExt, CompilerOptions, Plugin,
  PluginDriver, SharedPluginDriver,
};

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
    let plugin_driver = Arc::new(RwLock::new(PluginDriver::new(
      options.clone(),
      plugins,
      resolver_factory.clone(),
    )));
    let cache = Arc::new(Cache::new(options.clone()));

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        Default::default(),
        plugin_driver.clone(),
        resolver_factory.clone(),
        cache.clone(),
      ),
      output_filesystem,
      plugin_driver,
      resolver_factory,
      cache,
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
    self
      .plugin_driver
      .read()
      .await
      .resolver_factory
      .clear_entries();

    fast_set(
      &mut self.compilation,
      Compilation::new(
        // TODO: use Arc<T> instead
        self.options.clone(),
        self.options.entry.clone(),
        Default::default(),
        self.plugin_driver.clone(),
        self.resolver_factory.clone(),
        self.cache.clone(),
      ),
    );

    self.plugin_driver.write().await.before_compile().await?;

    // Fake this compilation as *currently* rebuilding does not create a new compilation
    self
      .plugin_driver
      .write()
      .await
      .this_compilation(&mut self.compilation)
      .await?;

    self
      .plugin_driver
      .write()
      .await
      .compilation(&mut self.compilation)
      .await?;

    self.compilation.setup_entry_dependencies();

    let deps = self
      .compilation
      .entry_dependencies
      .iter()
      .flat_map(|(_, deps)| {
        deps
          .clone()
          .into_iter()
          .map(|d| (d, None))
          .collect::<Vec<_>>()
      })
      .collect::<HashSet<_>>();
    self.compile(SetupMakeParam::ForceBuildDeps(deps)).await?;
    self.cache.begin_idle();
    self.compile_done().await?;
    Ok(())
  }

  #[instrument(name = "compile", skip_all)]
  async fn compile(&mut self, params: SetupMakeParam) -> Result<()> {
    let option = self.options.clone();
    self.compilation.make(params).await?;
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
        .map(|types| types.iter().any(|item| item == "module"))
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
      self.compilation.bailout_module_identifiers = analyze_result.bail_out_module_identifiers;
      self.compilation.side_effects_free_modules = analyze_result.side_effects_free_modules;
      self.compilation.module_item_map = analyze_result.module_item_map;
      if self.options.builtins.tree_shaking.enable()
        && self.options.optimization.side_effects.is_enable()
      {
        self.compilation.include_module_ids = analyze_result.include_module_ids;
      }
      for entry in &self.compilation.entry_module_identifiers {
        if let Some(analyze_results) = analyze_result.analyze_results.get(entry) {
          self
            .compilation
            .exports_info_map
            .insert(*entry, analyze_results.ordered_exports());
        }
      }
      self.compilation.optimize_analyze_result_map = analyze_result.analyze_results;
    }
    self.compilation.seal(self.plugin_driver.clone()).await?;

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.read().await.take_diagnostic();
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
      self
        .output_filesystem
        .remove_dir_all(&self.options.output.path)
        .await?;
    }

    self
      .plugin_driver
      .write()
      .await
      .emit(&mut self.compilation)
      .await?;
    let _ = self
      .compilation
      .assets()
      .iter()
      .map(|(filename, asset)| self.emit_asset(&self.options.output.path, filename, asset))
      .collect::<FuturesResults<_>>();

    self
      .plugin_driver
      .write()
      .await
      .after_emit(&mut self.compilation)
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

      // let file = File::create(file_path).map_err(rspack_error::Error::from)?;
      // let mut writer = BufWriter::new(file);
      // source
      //   .as_ref()
      //   .to_writer(&mut writer)
      //   .map_err(rspack_error::Error::from)?;

      self.compilation.emitted_assets.insert(filename.to_string());
    }
    Ok(())
  }
}
