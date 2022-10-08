mod compilation;
mod resolver;

pub use compilation::*;
pub use resolver::*;

use std::{path::Path, sync::Arc};

use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::{
  emitter::{DiagnosticDisplay, StdioDiagnosticDisplay},
  Error, Result, TWithDiagnosticArray,
};
use rspack_sources::BoxSource;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{
  CompilerOptions, Dependency, LoaderRunnerRunner, ModuleGraphModule, Plugin, PluginDriver,
  SharedPluginDriver, Stats, PATH_START_BYTE_POS_MAP,
};

#[derive(Debug)]
pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub compilation: Compilation,
  pub plugin_driver: SharedPluginDriver,
  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
}

impl Compiler {
  #[instrument(skip_all)]
  pub fn new(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let options = Arc::new(options);

    let resolver_factory = Arc::new(ResolverFactory::default());
    let resolver = resolver_factory.get(options.resolve.clone());
    let plugin_driver = Arc::new(RwLock::new(PluginDriver::new(
      options.clone(),
      plugins,
      Arc::new(resolver),
    )));
    let loader_runner_runner = Arc::new(LoaderRunnerRunner::new(
      options.clone(),
      resolver_factory,
      plugin_driver.clone(),
    ));

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        Default::default(),
        Default::default(),
        plugin_driver.clone(),
        loader_runner_runner.clone(),
      ),
      plugin_driver,
      loader_runner_runner,
    }
  }

  pub async fn rebuild(&mut self, _changed_files_path: Vec<String>) -> Result<Stats> {
    // let deps = changed_files_path.into_iter().map(|(name, specifier)| {
    //   (
    //     name.clone(),
    //     Dependency {
    //       importer: None,
    //       detail: ModuleDependency {
    //         specifier,
    //         kind: ResolveKind::Import,
    //         span: None,
    //       },
    //     },
    //   )
    // });
    // self.compile(deps).await?;
    self.stats()
  }
  pub async fn run(&mut self) -> anyhow::Result<()> {
    let stats = self.build().await?;
    if !stats.compilation.diagnostic.is_empty() {
      let err_msg = stats.emit_error_string(true).unwrap();
      anyhow::bail!(err_msg)
    }
    Ok(())
  }
  #[instrument(name = "build")]
  pub async fn build(&mut self) -> Result<Stats> {
    self.compilation = Compilation::new(
      // TODO: use Arc<T> instead
      self.options.clone(),
      self.options.entry.clone(),
      Default::default(),
      Default::default(),
      self.plugin_driver.clone(),
      self.loader_runner_runner.clone(),
    );
    let deps = self.compilation.entry_dependencies();
    self.compile(deps).await?;
    self.stats()
  }

  #[instrument(name = "compile")]
  async fn compile(&mut self, deps: HashMap<String, Dependency>) -> Result<()> {
    self.compilation.make(deps).await;
    self.compilation.seal(self.plugin_driver.clone()).await?;

    // Consume plugin driver diagnostic
    let mut plugin_driver_diagnostics = self.plugin_driver.read().await.take_diagnostic();
    self
      .compilation
      .diagnostic
      .append(&mut plugin_driver_diagnostics);

    self.emit_assets(&self.compilation)?;
    self.compilation.done(self.plugin_driver.clone()).await?;

    Ok(())
  }

  fn stats(&mut self) -> Result<Stats> {
    if self.options.emit_error {
      StdioDiagnosticDisplay::default().emit_batch_diagnostic(
        &self.compilation.diagnostic,
        PATH_START_BYTE_POS_MAP.clone(),
      )?;
    }
    Ok(Stats::new(&self.compilation))
  }

  pub fn emit_assets(&self, compilation: &Compilation) -> Result<()> {
    std::fs::create_dir_all(Path::new(&self.options.context).join(&self.options.output.path))
      .map_err(|_| Error::InternalError("Failed to create output directory".into()))?;

    std::fs::create_dir_all(&self.options.output.path)
      .map_err(|_| Error::InternalError("Failed to create output directory".into()))?;

    compilation
      .assets()
      .par_iter()
      .try_for_each(|(filename, asset)| -> anyhow::Result<()> {
        use std::fs;

        std::fs::create_dir_all(
          Path::new(&self.options.output.path)
            .join(filename)
            .parent()
            .unwrap(),
        )?;

        fs::write(
          Path::new(&self.options.output.path).join(filename),
          asset.buffer(),
        )
        .map_err(|e| e.into())
      })
      .map_err(|e| e.into())
  }

  pub fn update_asset(&mut self, filename: String, asset: BoxSource) {
    self.compilation.assets.insert(filename, asset);
    dbg!(
      "change",
      &self.compilation.assets.entry("main.js".to_owned())
    );
  }
}

#[derive(Debug)]
pub enum Msg {
  DependencyReference(Dependency, String),
  TaskFinished(TWithDiagnosticArray<Box<ModuleGraphModule>>),
  TaskCanceled,
  TaskErrorEncountered(Error),
}
