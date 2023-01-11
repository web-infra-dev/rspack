mod compilation;
mod hmr;
mod queue;
mod resolver;

use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use anyhow::Context;
pub use compilation::*;
pub use queue::*;
use rayon::prelude::*;
pub use resolver::*;
use rspack_error::{Error, Result};
use rustc_hash::FxHashSet as HashSet;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
  cache::Cache, fast_set, CompilerOptions, LoaderRunnerRunner, Plugin, PluginDriver,
  SharedPluginDriver,
};

#[derive(Debug)]
pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub compilation: Compilation,
  pub plugin_driver: SharedPluginDriver,
  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub cache: Arc<Cache>,
}

impl Compiler {
  #[instrument(skip_all)]
  pub fn new(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let options = Arc::new(options);

    let resolver_factory = Arc::new(ResolverFactory::new(options.resolve.clone()));
    let plugin_driver = Arc::new(RwLock::new(PluginDriver::new(
      options.clone(),
      plugins,
      resolver_factory.clone(),
    )));
    let loader_runner_runner = Arc::new(LoaderRunnerRunner::new(
      options.clone(),
      resolver_factory,
      plugin_driver.clone(),
    ));
    let cache = Arc::new(Cache::new(options.clone()));

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        Default::default(),
        plugin_driver.clone(),
        loader_runner_runner.clone(),
        cache.clone(),
      ),
      plugin_driver,
      loader_runner_runner,
      cache,
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    self.build().await?;
    Ok(())
  }

  #[instrument(name = "build", skip_all)]
  pub async fn build(&mut self) -> Result<()> {
    self.cache.end_idle().await;
    // TODO: clear the outdate cache entires in resolver,
    // TODO: maybe it's better to use external entries.

    fast_set(
      &mut self.compilation,
      Compilation::new(
        // TODO: use Arc<T> instead
        self.options.clone(),
        self.options.entry.clone(),
        Default::default(),
        self.plugin_driver.clone(),
        self.loader_runner_runner.clone(),
        self.cache.clone(),
      ),
    );

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
      .flat_map(|(_, deps)| deps.clone())
      .collect::<HashSet<_>>();
    self.compile(SetupMakeParam::ForceBuildDeps(deps)).await?;
    self.cache.begin_idle().await;

    #[cfg(debug_assertions)]
    {
      if self.options.__emit_error {
        let stats = crate::Stats::new(&self.compilation);
        stats
          .emit_diagnostics()
          .expect("failed to emit diagnostics");
      }
    }

    Ok(())
  }

  #[instrument(name = "compile", skip_all)]
  async fn compile(&mut self, params: SetupMakeParam) -> Result<()> {
    let option = self.options.clone();
    self.compilation.make(params).await?;
    if option.builtins.tree_shaking {
      let (analyze_result, diagnostics) = self
        .compilation
        .optimize_dependency()
        .await?
        .split_into_parts();
      if !diagnostics.is_empty() {
        self.compilation.push_batch_diagnostic(diagnostics);
      }
      self.compilation.used_symbol = analyze_result.used_direct_symbol;
      self.compilation.bailout_module_identifiers = analyze_result.bail_out_module_identifiers;
      self.compilation.used_indirect_symbol = analyze_result.used_indirect_symbol;

      // This is only used when testing
      #[cfg(debug_assertions)]
      {
        self.compilation.tree_shaking_result = analyze_result.analyze_results;
      }
    }
    self.compilation.seal(self.plugin_driver.clone()).await?;

    // Consume plugin driver diagnostic
    let plugin_driver_diagnostics = self.plugin_driver.read().await.take_diagnostic();
    self
      .compilation
      .push_batch_diagnostic(plugin_driver_diagnostics);

    if !self.compilation.options.builtins.no_emit_assets {
      self.emit_assets().await?;
    }

    self.compilation.done(self.plugin_driver.clone()).await?;

    Ok(())
  }

  #[instrument(name = "emit_assets", skip_all)]
  pub async fn emit_assets(&mut self) -> Result<()> {
    self
      .plugin_driver
      .write()
      .await
      .emit(&mut self.compilation)
      .await?;

    let output_path = self.options.context.join(&self.options.output.path);
    if !output_path.exists() {
      std::fs::create_dir_all(&output_path)
        .with_context(|| format!("failed to create dir: {:?}", &output_path))
        .map_err(|e| Error::Anyhow { source: e })?;
    }

    self
      .compilation
      .assets()
      .par_iter()
      .try_for_each(|(filename, asset)| self.emit_asset(&output_path, filename, asset))?;

    self
      .plugin_driver
      .write()
      .await
      .after_emit(&mut self.compilation)
      .await
  }

  fn emit_asset(&self, output_path: &Path, filename: &str, asset: &CompilationAsset) -> Result<()> {
    if let Some(source) = asset.get_source() {
      let file_path = Path::new(&output_path).join(filename);
      std::fs::create_dir_all(
        file_path
          .parent()
          .unwrap_or_else(|| panic!("The parent of {} can't found", file_path.display())),
      )?;
      let file = File::create(file_path).map_err(rspack_error::Error::from)?;
      let mut writer = BufWriter::new(file);
      source
        .as_ref()
        .to_writer(&mut writer)
        .map_err(rspack_error::Error::from)?;
      self.compilation.emitted_assets.insert(filename.to_string());
    }
    Ok(())
  }
}
