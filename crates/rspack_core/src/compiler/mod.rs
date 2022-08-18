use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use anyhow::Context;
use rayon::prelude::*;
use tracing::instrument;

use crate::{
  AssetContent, CompilerOptions, Dependency, LoaderRunnerRunner, ModuleGraphModule,
  NormalModuleFactory, NormalModuleFactoryContext, Plugin, PluginDriver, Stats,
};

mod compilation;
mod resolver;

pub use compilation::*;
pub use resolver::*;

pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub compilation: Compilation,
  pub plugin_driver: Arc<PluginDriver>,
  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
}

impl Compiler {
  #[instrument(skip_all)]
  pub fn new(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let options = Arc::new(options);

    let resolver_factory = ResolverFactory::new();
    let resolver = resolver_factory.get(options.resolve.clone());
    let plugin_driver = PluginDriver::new(options.clone(), plugins, Arc::new(resolver));
    let loader_runner_runner = LoaderRunnerRunner::new(options.clone());

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        Default::default(),
        Default::default(),
      ),
      plugin_driver: Arc::new(plugin_driver),
      loader_runner_runner: Arc::new(loader_runner_runner),
    }
  }

  #[instrument(skip_all)]
  pub async fn compile(&mut self) -> anyhow::Result<Stats> {
    // TODO: supports rebuild
    self.compilation = Compilation::new(
      // TODO: use Arc<T> instead
      self.options.clone(),
      self.options.entry.clone(),
      Default::default(),
      Default::default(),
    );

    // self.compilation.
    let active_task_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    self
      .compilation
      .entry_dependencies()
      .into_iter()
      .for_each(|(name, dep)| {
        let task = NormalModuleFactory::new(
          NormalModuleFactoryContext {
            module_name: Some(name),
            active_task_count: active_task_count.clone(),
            visited_module_identity: self.compilation.visited_module_id.clone(),
            module_type: None,
            side_effects: None,
            options: self.options.clone(),
          },
          dep,
          tx.clone(),
          self.plugin_driver.clone(),
          self.loader_runner_runner.clone(),
        );

        tokio::task::spawn(async move { task.run().await });
      });

    while active_task_count.load(Ordering::SeqCst) != 0 {
      match rx.recv().await {
        Some(job) => match job {
          Msg::TaskFinished(module) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            self.compilation.module_graph.add_module(*module);
          }
          Msg::TaskCanceled => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          Msg::DependencyReference(dep, resolved_uri) => {
            self
              .compilation
              .module_graph
              .add_dependency(dep, resolved_uri);
          }
          Msg::TaskErrorEncountered(err) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            return Err(err);
          }
        },
        None => {
          tracing::trace!("All sender is dropped");
        }
      }
    }

    tracing::debug!("module graph {:#?}", self.compilation.module_graph);

    // self.compilation.calc_exec_order();

    self.compilation.seal(self.plugin_driver.clone());

    // tracing::trace!("assets {:#?}", assets);

    std::fs::create_dir_all(Path::new(&self.options.context).join(&self.options.output.path))
      .context("failed to create output directory")?;

    std::fs::create_dir_all(&self.options.output.path)
      .context("failed to create output directory")?;

    self
      .compilation
      .assets
      .par_iter()
      .try_for_each(|(filename, asset)| -> anyhow::Result<()> {
        use std::fs;

        std::fs::create_dir_all(
          Path::new(&self.options.output.path)
            .join(filename)
            .parent()
            .unwrap(),
        )?;

        match asset.source() {
          AssetContent::Buffer(buf) => {
            fs::write(Path::new(&self.options.output.path).join(filename), buf)
              .context("failed to write asset")
          }
          AssetContent::String(str) => {
            fs::write(Path::new(&self.options.output.path).join(filename), str)
              .context("failed to write asset")
          }
        }
      })?;

    Ok(Stats::new(&self.compilation))
  }

  #[instrument(skip_all)]
  pub async fn run(&mut self) -> anyhow::Result<Stats> {
    self.compile().await
  }
}

#[derive(Debug)]
pub enum Msg {
  DependencyReference(Dependency, String),
  TaskFinished(Box<ModuleGraphModule>),
  TaskCanceled,
  TaskErrorEncountered(anyhow::Error),
}
