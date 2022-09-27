use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
  time::Instant,
};

use crate::{
  CompilerOptions, Dependency, LoaderRunnerRunner, ModuleGraphModule, NormalModuleFactory,
  NormalModuleFactoryContext, Plugin, PluginDriver, Stats, PATH_START_BYTE_POS_MAP,
};

use anyhow::Context;
use crossbeam::queue::SegQueue;
use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::{
  emitter::{DiagnosticDisplay, StdioDiagnosticDisplay},
  Error, Result, TWithDiagnosticArray,
};
use rspack_sources::BoxSource;
use tokio::runtime::Builder;
use tracing::instrument;

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
    let plugin_driver = Arc::new(PluginDriver::new(
      options.clone(),
      plugins,
      Arc::new(resolver),
    ));
    let loader_runner_runner = LoaderRunnerRunner::new(options.clone(), plugin_driver.clone());

    Self {
      options: options.clone(),
      compilation: Compilation::new(
        options,
        Default::default(),
        Default::default(),
        Default::default(),
      ),
      plugin_driver,
      loader_runner_runner: Arc::new(loader_runner_runner),
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
  pub async fn build(&mut self) -> Result<Stats> {
    self.compilation = Compilation::new(
      // TODO: use Arc<T> instead
      self.options.clone(),
      self.options.entry.clone(),
      Default::default(),
      Default::default(),
    );
    let deps = self.compilation.entry_dependencies();
    self.compile(deps).await?;
    self.stats()
  }

  #[instrument(skip_all)]
  async fn compile(&mut self, deps: HashMap<String, Dependency>) -> Result<()> {
    let start = Instant::now();
    let thread_count = 16;
    let active_task_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(thread_count));

    let queue = Arc::new(SegQueue::<NormalModuleFactory>::new());

    let (tx, rx) = crossbeam::channel::unbounded::<Msg>();

    deps.into_iter().for_each(|(name, dep)| {
      let task = NormalModuleFactory::new(
        NormalModuleFactoryContext {
          module_name: Some(name),
          module_queue: queue.clone(),
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
      queue.push(task);
    });

    let mut quit_thread_count = 0;
    for i in 0..thread_count {
      let active_task_count = active_task_count.clone();
      let tx = tx.clone();
      let queue = queue.clone();
      std::thread::spawn(move || {
        let rt = Builder::new_current_thread().build().unwrap();

        rt.block_on(async {
          loop {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            if let Some(task) = queue.pop() {
              task.run().await;
              active_task_count.fetch_add(1, Ordering::SeqCst);
            } else {
              active_task_count.fetch_add(1, Ordering::SeqCst);
              loop {
                if !queue.is_empty() {
                  break;
                } else if active_task_count.load(Ordering::SeqCst) == thread_count {
                  tx.send(Msg::ThreadQuit).unwrap();
                  return;
                }
              }
            }
          }
        });
      });
    }
    // while active_task_count.load(Ordering::SeqCst) != 0 {
    loop {
      match rx.try_recv() {
        Ok(job) => match job {
          Msg::TaskFinished(mut module_with_diagnostic) => {
            // active_task_count.fetch_sub(1, Ordering::SeqCst);
            self
              .compilation
              .module_graph
              .add_module(*module_with_diagnostic.inner);
            self
              .compilation
              .diagnostic
              .append(&mut module_with_diagnostic.diagnostic);
          }
          Msg::TaskCanceled => {
            // active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          Msg::DependencyReference(dep, resolved_uri) => {
            self
              .compilation
              .module_graph
              .add_dependency(dep, resolved_uri);
          }
          Msg::TaskErrorEncountered(err) => {
            // active_task_count.fetch_sub(1, Ordering::SeqCst);
            self.compilation.push_batch_diagnostic(err.into());
          }
          Msg::ThreadQuit => {
            quit_thread_count += 1;
            // println!("quit ");
          }
        },
        Err(_) => {
          if quit_thread_count == thread_count {
            break;
          }
          tracing::trace!("All sender is dropped");
        }
      }
    }
    // }

    println!("{:?}", start.elapsed());
    tracing::debug!("module graph {:#?}", self.compilation.module_graph);

    // self.compilation.calc_exec_order();
    let start = Instant::now();
    self.compilation.seal(self.plugin_driver.clone()).await?;
    println!("{:?}", start.elapsed());
    // Consume plugin driver diagnostic
    let mut plugin_driver_diagnostics = self.plugin_driver.take_diagnostic();
    self
      .compilation
      .diagnostic
      .append(&mut plugin_driver_diagnostics);

    // tracing::trace!("assets {:#?}", assets);

    std::fs::create_dir_all(Path::new(&self.options.context).join(&self.options.output.path))
      .map_err(|_| Error::InternalError("failed to create output directory".into()))?;

    std::fs::create_dir_all(&self.options.output.path)
      .map_err(|_| Error::InternalError("failed to create output directory".into()))?;
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

        fs::write(
          Path::new(&self.options.output.path).join(filename),
          asset.buffer(),
        )
        .context("failed to write asset")
      })
      .unwrap();
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
  ThreadQuit,
}
