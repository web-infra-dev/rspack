use std::{collections::VecDeque, path::Path, sync::Arc};

use crate::{
  CompilerOptions, Dependency, LoaderRunnerRunner, ModuleGraphModule, NormalModuleFactory,
  NormalModuleFactoryContext, Plugin, PluginDriver, Stats, PATH_START_BYTE_POS_MAP,
};
use anyhow::Context;
use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::{
  emitter::{DiagnosticDisplay, StdioDiagnosticDisplay},
  Error, Result, TWithDiagnosticArray,
};
use rspack_sources::BoxSource;
use tracing::instrument;

mod compilation;
mod resolver;

pub use compilation::*;
pub use resolver::*;

#[derive(Debug)]
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

    let resolver_factory = Arc::new(ResolverFactory::default());
    let resolver = resolver_factory.get(options.resolve.clone());
    let plugin_driver = Arc::new(PluginDriver::new(
      options.clone(),
      plugins,
      Arc::new(resolver),
    ));
    let loader_runner_runner =
      LoaderRunnerRunner::new(options.clone(), resolver_factory, plugin_driver.clone());

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

  async fn make(&mut self, deps: HashMap<String, Dependency>) {
    let mut queue = VecDeque::new();

    deps.into_iter().for_each(|(name, dep)| {
      let task = NormalModuleFactory::new(
        NormalModuleFactoryContext {
          module_name: Some(name),
          visited_module_identity: self.compilation.visited_module_id.clone(),
          module_type: None,
          side_effects: None,
          options: self.options.clone(),
        },
        dep,
        self.plugin_driver.clone(),
        self.loader_runner_runner.clone(),
      );
      queue.push_back(task);
    });

    while let Some(mut task) = queue.pop_front() {
      let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

      match task.pre_resolve_module() {
        Ok(pre_result) => match pre_result {
          Some((uri, module)) => {
            self
              .compilation
              .module_graph
              .add_dependency(task.dependency.clone(), uri.clone());

            tokio::task::spawn(async move {
              match task.resolve_module(uri, module).await {
                Ok((next_tasks, resolved_module)) => {
                  let diagnostic = std::mem::take(&mut task.diagnostic);
                  if let Err(err) = tx.send(Msg::TaskFinished((
                    next_tasks,
                    TWithDiagnosticArray::new(Box::new(resolved_module), diagnostic),
                  ))) {
                    tracing::trace!("fail to send msg {:?}", err)
                  }
                }
                Err(error) => {
                  if let Err(err) = tx.send(Msg::TaskErrorEncountered(error)) {
                    tracing::trace!("fail to send msg {:?}", err)
                  }
                }
              };
            });
          }
          None => continue,
        },
        Err(error) => {
          self.compilation.push_batch_diagnostic(error.into());
        }
      }

      while let Some(job) = rx.recv().await {
        match job {
          Msg::DependencyReference(_, _) => (),
          Msg::TaskFinished((mut next_tasks, mut module_with_diagnostic)) => {
            queue.append(&mut next_tasks.into());
            self
              .compilation
              .module_graph
              .add_module(*module_with_diagnostic.inner);
            self
              .compilation
              .diagnostic
              .append(&mut module_with_diagnostic.diagnostic);
          }
          Msg::TaskCanceled => {}
          Msg::TaskErrorEncountered(err) => {
            self.compilation.push_batch_diagnostic(err.into());
          }
        }
      }
    }

    // while active_task_count.load(Ordering::SeqCst) != 0 {
    //   match rx.recv().await {
    //     Some(job) => match job {
    //       Msg::TaskFinished(mut module_with_diagnostic) => {
    //         active_task_count.fetch_sub(1, Ordering::SeqCst);
    //         self
    //           .compilation
    //           .module_graph
    //           .add_module(*module_with_diagnostic.inner);
    //         self
    //           .compilation
    //           .diagnostic
    //           .append(&mut module_with_diagnostic.diagnostic);
    //       }
    //       Msg::TaskCanceled => {
    //         active_task_count.fetch_sub(1, Ordering::SeqCst);
    //       }
    //       Msg::TaskErrorEncountered(err) => {
    //         active_task_count.fetch_sub(1, Ordering::SeqCst);
    //         self.compilation.push_batch_diagnostic(err.into());
    //       }
    //     },
    //     None => {
    //       tracing::trace!("All sender is dropped");
    //     }
    //   }
    // }
    tracing::debug!("module graph {:#?}", self.compilation.module_graph);
  }
  #[instrument(name = "compile")]
  async fn compile(&mut self, deps: HashMap<String, Dependency>) -> Result<()> {
    self.make(deps).await;
    self.compilation.seal(self.plugin_driver.clone()).await?;

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
  TaskFinished(
    (
      Vec<NormalModuleFactory>,
      TWithDiagnosticArray<Box<ModuleGraphModule>>,
    ),
  ),
  TaskCanceled,
  TaskErrorEncountered(Error),
}
