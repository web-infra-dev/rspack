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
  CompilerOptions, Dependency, LoaderRunnerRunner, ModuleGraphModule, ModuleIdentifier,
  NormalModule, Plugin, PluginDriver, SharedPluginDriver, Stats, PATH_START_BYTE_POS_MAP,
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
    // TODO: clear the outdate cache entires in resolver,
    // TODO: maybe it's better to use external entries.
    self.plugin_driver.read().await.resolver.clear();

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

  fn stats(&self) -> Result<Stats> {
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

  // TODO: remove this function when we had hash in stats.
  pub async fn rebuild(&mut self) -> Result<std::collections::HashMap<String, (u8, String)>> {
    fn collect_modules_from_stats(s: &Stats<'_>) -> HashMap<String, String> {
      let modules = s.compilation.module_graph.modules();
      let modules = modules.filter(|item| item.module_type.is_js_like());
      // TODO: use hash;
      modules
        .map(|item| {
          let uri = item.uri.to_string();
          // TODO: it soo slowly, should use cache to instead.
          let code = item.module.code_generation(item, s.compilation).unwrap();
          let code = code
            .inner()
            .get(&crate::SourceType::JavaScript)
            .expect("expected javascript file")
            .ast_or_source
            .as_source()
            .unwrap()
            .source()
            .to_string();
          (uri, code)
        })
        .collect()
    }

    let old = self.stats()?;
    let old = collect_modules_from_stats(&old);

    let new = self.build().await?;
    let new = collect_modules_from_stats(&new);

    let mut diff = std::collections::HashMap::new();

    for (new_uri, new_content) in &new {
      if !old.contains_key(new_uri) {
        // added
        diff.insert(new_uri.to_string(), (2, new_content.to_string()));
      }
    }

    for (old_uri, old_content) in old {
      if let Some(new_content) = new.get(&old_uri) {
        // changed
        // TODO: should use module hash.
        if *new_content != old_content {
          diff.insert(old_uri, (0, new_content.to_string()));
        }
      } else {
        // deleted
        diff.insert(old_uri, (1, String::new()));
      }
    }

    Ok(diff)
  }
}

#[derive(Debug)]
pub enum Msg {
  DependencyReference(Dependency, String),
  TaskFinished(
    TWithDiagnosticArray<
      Box<(
        ModuleGraphModule,
        NormalModule,
        ModuleIdentifier,
        ModuleDependency,
      )>,
    >,
  ),
  TaskCanceled,
  TaskErrorEncountered(Error),
}
