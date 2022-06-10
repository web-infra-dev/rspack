use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc,
};

use nodejs_resolver::Resolver;

use crate::{
  CompilerOptions, Dependency, JobContext, ModuleGraphModule, Plugin, PluginDriver,
  RenderManifestArgs, ResolvingModuleJob,
};

mod compilation;
pub use compilation::*;

pub struct Compiler {
  pub options: Arc<CompilerOptions>,
  pub compilation: Compilation,
  pub plugin_driver: Arc<PluginDriver>,
}

impl Compiler {
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

  pub async fn compile(&mut self) -> anyhow::Result<()> {
    // TODO: supports rebuild
    self.compilation = Compilation::new(
      // TODO: use Arc<T> instead
      self.options.clone(),
      self.options.entries.clone(),
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
      .for_each(|dep| {
        let task = ResolvingModuleJob::new(
          JobContext {
            importer: None,
            active_task_count: active_task_count.clone(),
            visited_module_uri: self.compilation.visited_module_id.clone(),
            source_type: None,
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

    self.compilation.calc_exec_order();

    self.compilation.seal();

    tracing::debug!("chunk graph {:#?}", self.compilation.chunk_graph);

    let assets = self
      .compilation
      .chunk_graph
      .id_to_chunk()
      .keys()
      .flat_map(|chunk_id| {
        self.plugin_driver.render_manifest(RenderManifestArgs {
          chunk_id,
          compilation: &self.compilation,
        })
      })
      .collect::<Vec<_>>();

    tracing::trace!("assets {:#?}", assets);
    Ok(())
  }

  pub async fn run(&mut self) -> anyhow::Result<()> {
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
