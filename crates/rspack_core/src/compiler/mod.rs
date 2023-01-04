mod compilation;
mod hmr;
mod queue;
mod resolver;

use anyhow::Context;
pub use compilation::*;
pub use queue::*;
pub use resolver::*;
use std::{path::Path, sync::Arc};

use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::{Error, Result};
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
  cache::Cache, fast_set, CompilerOptions, Dependency, LoaderRunnerRunner, Plugin, PluginDriver,
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
    let resolver = resolver_factory.get(options.resolve.clone());
    let plugin_driver = Arc::new(RwLock::new(PluginDriver::new(
      options.clone(),
      plugins,
      resolver,
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
    self.plugin_driver.read().await.resolver.clear();

    fast_set(
      &mut self.compilation,
      Compilation::new(
        // TODO: use Arc<T> instead
        self.options.clone(),
        self.options.entry.clone(),
        Default::default(),
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

    let deps = self.compilation.entry_dependencies();
    self.compile(deps).await?;
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
  async fn compile(&mut self, deps: HashMap<String, Vec<Dependency>>) -> Result<()> {
    let option = self.options.clone();
    self.compilation.make(deps).await;
    if option.builtins.tree_shaking {
      let (analyze_result, diagnostics) = self
        .compilation
        .optimize_dependency()
        .await?
        .split_into_parts();
      if !diagnostics.is_empty() {
        self.compilation.push_batch_diagnostic(diagnostics);
      }
      dbg!(&analyze_result.used_symbol.len());
      dbg!(&analyze_result.bail_out_module_identifiers.len());
      dbg!(&analyze_result.used_indirect_symbol.len());
      self.compilation.used_symbol = analyze_result.used_symbol;
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
      self.emit_assets(&self.compilation)?;
    }

    self.compilation.done(self.plugin_driver.clone()).await?;

    Ok(())
  }

  #[instrument(name = "emit_assets", skip_all)]
  pub fn emit_assets(&self, compilation: &Compilation) -> Result<()> {
    let output_path = self.options.context.join(&self.options.output.path);
    if !output_path.exists() {
      std::fs::create_dir_all(&output_path)
        .with_context(|| format!("failed to create dir: {:?}", &output_path))
        .map_err(|e| Error::Anyhow { source: e })?;
    }

    compilation
      .assets()
      .par_iter()
      .try_for_each(|(filename, asset)| self.emit_asset(&output_path, filename, asset))
  }

  #[instrument(name = "emit_asset", skip_all)]
  fn emit_asset(&self, output_path: &Path, filename: &str, asset: &CompilationAsset) -> Result<()> {
    let file_path = Path::new(&output_path).join(filename);
    std::fs::create_dir_all(
      file_path
        .parent()
        .unwrap_or_else(|| panic!("The parent of {} can't found", file_path.display())),
    )?;
    std::fs::write(file_path, asset.get_source().buffer()).map_err(rspack_error::Error::from)?;
    self.compilation.emitted_assets.insert(filename.to_string());
    Ok(())
  }
}
