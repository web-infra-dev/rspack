use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use anyhow::Context;
use nodejs_resolver::Resolver;
use rayon::prelude::*;
use tracing::instrument;

use crate::{
  Asset, AssetContent, CompilerOptions, Dependency, ModuleGraphModule, NormalModuleFactory,
  NormalModuleFactoryContext, Plugin, PluginDriver, RenderManifestArgs, SourceType, Stats,
};

mod compilation;
pub use compilation::*;

pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub compilation: Compilation,
  pub plugin_driver: Arc<PluginDriver>,
}

impl Compiler {
  #[instrument(skip_all)]
  pub fn new(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let options = Arc::new(options);
    let plugin_driver = PluginDriver::new(
      options.clone(),
      plugins,
      Arc::new(Resolver::new(nodejs_resolver::ResolverOptions {
        // prefer_relative: false,
        extensions: vec![".tsx", ".jsx", ".ts", ".js", ".json"]
          .into_iter()
          .map(|s| s.to_string())
          .collect(),
        alias_fields: vec![String::from("browser")],
        ..Default::default()
      })),
    );

    Self {
      options,
      compilation: Default::default(),
      plugin_driver: Arc::new(plugin_driver),
    }
  }

  #[instrument(skip_all)]
  pub async fn compile(&mut self) -> anyhow::Result<Stats> {
    let final_out_dir = {
      let root = Path::new(self.options.root.as_str());
      let out_dir = Path::new("./dist");
      root.join(out_dir)
    };

    // TODO: supports rebuild
    self.compilation = Compilation::new(
      // TODO: use Arc<T> instead
      self.options.clone(),
      self.options.entries.clone(),
      Default::default(),
      Default::default(),
      final_out_dir.clone(),
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
          },
          dep,
          tx.clone(),
          self.plugin_driver.clone(),
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

    self.compilation.seal();
    self.compilation.chunk_graph.chunks_mut().for_each(|chunk| {
      chunk.calc_exec_order(&self.compilation.module_graph);
    });

    tracing::debug!("chunk graph {:#?}", self.compilation.chunk_graph);
    // Stream::
    let assets = self.compilation.assets(self.plugin_driver.clone());

    // tracing::trace!("assets {:#?}", assets);

    std::fs::create_dir_all(&final_out_dir).unwrap();

    assets
      .par_iter()
      .try_for_each(|asset| -> anyhow::Result<()> {
        use std::fs;
        let filename = asset.filename();

        std::fs::create_dir_all(final_out_dir.join(filename).parent().unwrap())?;

        match &asset.content() {
          AssetContent::Buffer(buf) => {
            fs::write(final_out_dir.join(filename), buf).context("failed to write asset")
          }
          AssetContent::String(str) => {
            fs::write(final_out_dir.join(filename), str).context("failed to write asset")
          }
        }
      })?;

    Ok(Stats::new(assets))
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
