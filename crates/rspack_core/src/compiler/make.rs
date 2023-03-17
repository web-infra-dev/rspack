use std::{
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use rayon::prelude::{ParallelBridge, ParallelIterator};
use rspack_error::{CatchUnwindFuture, Diagnostic, Result};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::sync::mpsc::error::TryRecvError;

use super::{Compilation, SetupMakeParam};
use crate::{
  tree_shaking::BailoutFlag, utils::fast_drop, AddQueue, AddTask, AddTaskResult,
  BoxModuleDependency, BuildQueue, BuildTask, BuildTaskResult, CleanQueue, CleanTask,
  CleanTaskResult, DependencyId, FactorizeQueue, FactorizeTask, FactorizeTaskResult, Module,
  ModuleGraph, ModuleIdentifier, ModuleType, NormalModuleAstOrSource, ProcessDependenciesQueue,
  ProcessDependenciesResult, ProcessDependenciesTask, Resolve, SharedPluginDriver, TaskResult,
  WorkerTask,
};

pub struct CompilationMake<'a> {
  compilation: &'a mut Compilation,
  params: &'a SetupMakeParam,
}

impl<'a> CompilationMake<'a> {
  pub fn new(compilation: &'a mut Compilation, params: &'a SetupMakeParam) -> Self {
    Self {
      compilation,
      params,
    }
  }

  fn plugin_driver(&self) -> &SharedPluginDriver {
    &self.compilation.plugin_driver
  }

  fn module_graph(&self) -> &ModuleGraph {
    &self.compilation.module_graph
  }

  fn module_graph_mut(&mut self) -> &mut ModuleGraph {
    &mut self.compilation.module_graph
  }

  fn push_batch_diagnostic(&mut self, diagnostics: Vec<Diagnostic>) {
    self.compilation.push_batch_diagnostic(diagnostics);
  }

  pub async fn build(mut self) -> Result<()> {
    if let Some(e) = self
      .plugin_driver()
      .clone()
      .read()
      .await
      .make(self.compilation)
      .await
      .err()
    {
      self.push_batch_diagnostic(e.into());
    }

    // remove prev build ast in modules
    fast_drop(
      self
        .module_graph_mut()
        .modules_mut()
        .values_mut()
        .map(|module| {
          if let Some(m) = module.as_normal_module_mut() {
            let is_ast_unbuild = matches!(m.ast_or_source(), NormalModuleAstOrSource::Unbuild);
            if !is_ast_unbuild {
              return Some(std::mem::replace(
                m.ast_or_source_mut(),
                NormalModuleAstOrSource::Unbuild,
              ));
            }
          }
          None
        })
        .collect::<Vec<Option<NormalModuleAstOrSource>>>(),
    );

    let mut force_build_module = HashSet::default();
    let mut force_build_deps = std::mem::take(&mut self.compilation.make_failed_dependencies);
    let mut origin_module_deps = HashMap::default();
    // handle setup params
    if let SetupMakeParam::ModifiedFiles(files) = &self.params {
      force_build_module.extend(self.module_graph().modules().values().filter_map(|module| {
        // check has dependencies modified
        if self
          .module_graph()
          .has_dependencies(&module.identifier(), files)
        {
          Some(module.identifier())
        } else {
          None
        }
      }));
      // collect origin_module_deps
      for module_id in &force_build_module {
        let mgm = self
          .module_graph()
          .module_graph_module_by_identifier(module_id)
          .expect("module graph module not exist");
        let deps = mgm
          .all_depended_modules(self.module_graph())
          .into_iter()
          .cloned()
          .collect::<Vec<_>>();
        origin_module_deps.insert(*module_id, deps);
      }
    }
    if let SetupMakeParam::ForceBuildDeps(deps) = self.params {
      force_build_deps.extend(deps);
    }

    // move deps bindings module to force_build_module
    for dependency_id in &force_build_deps {
      if let Some(mid) = self
        .module_graph()
        .module_identifier_by_dependency_id(dependency_id)
      {
        force_build_module.insert(*mid);
      }
    }

    let mut need_check_isolated_module_ids = HashSet::default();
    let mut origin_module_issuers = HashMap::default();
    // calc need_check_isolated_module_ids & regen_module_issues
    for id in &force_build_module {
      if let Some(mgm) = self.module_graph().module_graph_module_by_identifier(id) {
        let depended_modules = mgm
          .all_depended_modules(self.module_graph())
          .into_iter()
          .copied();
        need_check_isolated_module_ids.extend(depended_modules);
        origin_module_issuers.insert(*id, mgm.get_issuer().clone());
      }
    }

    let mut active_task_count = 0usize;
    let is_expected_shutdown = Arc::new(AtomicBool::new(false));
    let (result_tx, mut result_rx) = tokio::sync::mpsc::unbounded_channel::<Result<TaskResult>>();
    let mut factorize_queue = FactorizeQueue::new();
    let mut add_queue = AddQueue::new();
    let mut build_queue = BuildQueue::new();
    let mut process_dependencies_queue = ProcessDependenciesQueue::new();
    let mut make_failed_dependencies: HashSet<DependencyId> = HashSet::default();
    let mut errored = None;

    force_build_deps.extend(
      force_build_module
        .iter()
        .flat_map(|id| self.module_graph_mut().revoke_module(id)),
    );

    force_build_deps.iter().for_each(|id| {
      let dependency = self
        .module_graph()
        .dependency_by_id(id)
        .expect("dependency not found");
      let parent_module_identifier = dependency.parent_module_identifier().cloned();
      let parent_module =
        parent_module_identifier.and_then(|id| self.module_graph().module_by_identifier(&id));
      if parent_module_identifier.is_some() && parent_module.is_none() {
        return;
      }

      self.handle_module_creation(
        &mut factorize_queue,
        parent_module_identifier,
        {
          parent_module
            .and_then(|m| m.as_normal_module())
            .map(|module| module.resource_resolved_data().resource_path.clone())
        },
        vec![dependency.clone()],
        parent_module_identifier.is_none(),
        None,
        None,
        parent_module.and_then(|module| module.get_resolve_options().map(ToOwned::to_owned)),
        self.compilation.lazy_visit_modules.clone(),
        parent_module
          .and_then(|m| m.as_normal_module())
          .and_then(|module| module.name_for_condition())
          .map(|issuer| issuer.to_string()),
      );
    });

    tokio::task::block_in_place(|| loop {
      while let Some(task) = factorize_queue.get_task() {
        tokio::spawn({
          let result_tx = result_tx.clone();
          let is_expected_shutdown = is_expected_shutdown.clone();
          active_task_count += 1;

          async move {
            if is_expected_shutdown.load(Ordering::SeqCst) {
              return;
            }

            let result = CatchUnwindFuture::create(task.run()).await;

            match result {
              Ok(result) => {
                if !is_expected_shutdown.load(Ordering::SeqCst) {
                  result_tx
                    .send(result)
                    .expect("Failed to send factorize result");
                }
              }
              Err(e) => {
                // panic on the tokio worker thread
                result_tx.send(Err(e)).expect("Failed to send panic info");
              }
            }
          }
        });
      }

      while let Some(task) = build_queue.get_task() {
        tokio::spawn({
          let result_tx = result_tx.clone();
          let is_expected_shutdown = is_expected_shutdown.clone();
          active_task_count += 1;

          async move {
            if is_expected_shutdown.load(Ordering::SeqCst) {
              return;
            }

            let result = CatchUnwindFuture::create(task.run()).await;

            match result {
              Ok(result) => {
                if !is_expected_shutdown.load(Ordering::SeqCst) {
                  result_tx.send(result).expect("Failed to send build result");
                }
              }
              Err(e) => {
                // panic on the tokio worker thread
                result_tx.send(Err(e)).expect("Failed to send panic info");
              }
            }
          }
        });
      }

      while let Some(task) = add_queue.get_task() {
        active_task_count += 1;
        let result = task.run(self.compilation);
        result_tx.send(result).expect("Failed to send add result");
      }

      while let Some(task) = process_dependencies_queue.get_task() {
        active_task_count += 1;

        task.dependencies.into_iter().for_each(|id| {
          let original_module_identifier = &task.original_module_identifier;
          let module = self
            .module_graph()
            .module_by_identifier(original_module_identifier)
            .expect("Module expected");
          let dependency = self
            .module_graph()
            .dependency_by_id(&id)
            .expect("dependency expected");

          self.handle_module_creation(
            &mut factorize_queue,
            Some(task.original_module_identifier),
            {
              module
                .as_normal_module()
                .map(|module| module.resource_resolved_data().resource_path.clone())
            },
            vec![dependency.clone()],
            false,
            None,
            None,
            task.resolve_options.clone(),
            self.compilation.lazy_visit_modules.clone(),
            module
              .as_normal_module()
              .and_then(|module| module.name_for_condition())
              .map(|issuer| issuer.to_string()),
          );
        });

        result_tx
          .send(Ok(TaskResult::ProcessDependencies(
            ProcessDependenciesResult {
              module_identifier: task.original_module_identifier,
            },
          )))
          .expect("Failed to send process dependencies result");
      }

      match result_rx.try_recv() {
        Ok(item) => {
          match item {
            Ok(TaskResult::Factorize(task_result)) => {
              let FactorizeTaskResult {
                is_entry,
                original_module_identifier,
                factory_result,
                module_graph_module,
                diagnostics,
                dependencies,
              } = task_result;

              tracing::trace!("Module created: {}", factory_result.module.identifier());
              if !diagnostics.is_empty() {
                make_failed_dependencies.insert(dependencies[0]);
              }

              self.push_batch_diagnostic(diagnostics);

              self
                .compilation
                .file_dependencies
                .extend(factory_result.file_dependencies);
              self
                .compilation
                .context_dependencies
                .extend(factory_result.context_dependencies);
              self
                .compilation
                .missing_dependencies
                .extend(factory_result.missing_dependencies);

              add_queue.add_task(AddTask {
                original_module_identifier,
                module: factory_result.module,
                module_graph_module,
                dependencies,
                is_entry,
              });
            }
            Ok(TaskResult::Add(task_result)) => match task_result {
              AddTaskResult::ModuleAdded {
                module,
                dependencies,
              } => {
                tracing::trace!("Module added: {}", module.identifier());
                build_queue.add_task(BuildTask {
                  module,
                  dependencies,
                  loader_runner_runner: self.compilation.loader_runner_runner.clone(),
                  compiler_options: self.compilation.options.clone(),
                  plugin_driver: self.plugin_driver().clone(),
                  cache: self.compilation.cache.clone(),
                });
              }
              AddTaskResult::ModuleReused { module, .. } => {
                tracing::trace!("Module reused: {}, skipping build", module.identifier());
              }
            },
            Ok(TaskResult::Build(task_result)) => {
              let BuildTaskResult {
                module,
                dependencies,
                build_result,
                diagnostics,
              } = task_result;

              if !diagnostics.is_empty() {
                make_failed_dependencies.insert(dependencies[0]);
              }

              tracing::trace!("Module built: {}", module.identifier());
              self.push_batch_diagnostic(diagnostics);

              self
                .compilation
                .file_dependencies
                .extend(build_result.build_info.file_dependencies.clone());
              self
                .compilation
                .context_dependencies
                .extend(build_result.build_info.context_dependencies.clone());
              self
                .compilation
                .missing_dependencies
                .extend(build_result.build_info.missing_dependencies.clone());
              self
                .compilation
                .build_dependencies
                .extend(build_result.build_info.build_dependencies.clone());

              let mut dep_ids = vec![];
              for dependency in build_result.dependencies {
                let dep_id = self.module_graph_mut().add_dependency(dependency);
                dep_ids.push(dep_id);
              }

              {
                let mgm = self
                  .module_graph_mut()
                  .module_graph_module_by_identifier_mut(&module.identifier())
                  .expect("Failed to get mgm");
                mgm.dependencies = dep_ids.clone();
              }
              process_dependencies_queue.add_task(ProcessDependenciesTask {
                dependencies: dep_ids.clone(),
                original_module_identifier: module.identifier(),
                resolve_options: module.get_resolve_options().map(ToOwned::to_owned),
              });
              self.module_graph_mut().set_module_build_info_and_meta(
                &module.identifier(),
                build_result.build_info,
                build_result.build_meta,
              );
              self.module_graph_mut().add_module(module);
            }
            Ok(TaskResult::ProcessDependencies(task_result)) => {
              tracing::trace!(
                "Processing dependencies of {} finished",
                task_result.module_identifier
              );
            }
            Err(err) => {
              // Severe internal error encountered, we should end the compiling here.
              errored = Some(err);
              is_expected_shutdown.store(true, Ordering::SeqCst);
              break;
            }
          }

          active_task_count -= 1;
        }
        Err(TryRecvError::Disconnected) => {
          is_expected_shutdown.store(true, Ordering::SeqCst);
          break;
        }
        Err(TryRecvError::Empty) => {
          if active_task_count == 0 {
            is_expected_shutdown.store(true, Ordering::SeqCst);
            break;
          }
        }
      }
    });

    self.compilation.make_failed_dependencies = make_failed_dependencies;
    tracing::debug!("All task is finished");

    // clean isolated module
    let mut clean_queue = CleanQueue::new();
    clean_queue.add_tasks(
      need_check_isolated_module_ids
        .into_iter()
        .map(|module_identifier| CleanTask { module_identifier }),
    );

    while let Some(task) = clean_queue.get_task() {
      match task.run(self.compilation) {
        CleanTaskResult::ModuleIsUsed { module_identifier } => {
          tracing::trace!("Module is used: {}", module_identifier);
        }
        CleanTaskResult::ModuleIsCleaned {
          module_identifier,
          dependent_module_identifiers,
        } => {
          tracing::trace!("Module is cleaned: {}", module_identifier);
          clean_queue.add_tasks(
            dependent_module_identifiers
              .into_iter()
              .map(|module_identifier| CleanTask { module_identifier }),
          );
        }
      };
    }

    tracing::debug!("All clean task is finished");
    // set origin module issues
    for (id, issuer) in origin_module_issuers {
      if let Some(mgm) = self
        .module_graph_mut()
        .module_graph_module_by_identifier_mut(&id)
      {
        mgm.set_issuer(issuer);
      }
    }

    // calc has_module_import_export_change
    self.compilation.has_module_import_export_change = if origin_module_deps.is_empty() {
      true
    } else {
      !origin_module_deps.into_iter().all(|(module_id, deps)| {
        if let Some(mgm) = self
          .module_graph()
          .module_graph_module_by_identifier(&module_id)
        {
          mgm.all_depended_modules(self.module_graph()) == deps.iter().collect::<Vec<_>>()
        } else {
          false
        }
      })
    };

    // add context module and context element module to bailout_module_identifiers
    if self.compilation.options.builtins.tree_shaking {
      self.compilation.bailout_module_identifiers = self
        .module_graph()
        .modules()
        .values()
        .par_bridge()
        .filter_map(|module| {
          if module.as_context_module().is_some() {
            let mut values = vec![(module.identifier(), BailoutFlag::CONTEXT_MODULE)];
            if let Some(dependencies) = self
              .module_graph()
              .dependencies_by_module_identifier(&module.identifier())
            {
              for dependency in dependencies {
                if let Some(dependency_module) = self
                  .module_graph()
                  .module_identifier_by_dependency_id(dependency)
                {
                  values.push((*dependency_module, BailoutFlag::CONTEXT_MODULE));
                }
              }
            }

            Some(values)
          } else {
            None
          }
        })
        .flatten()
        .collect();
    }

    if let Some(err) = errored {
      Err(err)
    } else {
      Ok(())
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn handle_module_creation(
    &self,
    queue: &mut FactorizeQueue,
    original_module_identifier: Option<ModuleIdentifier>,
    original_resource_path: Option<PathBuf>,
    dependencies: Vec<BoxModuleDependency>,
    is_entry: bool,
    module_type: Option<ModuleType>,
    side_effects: Option<bool>,
    resolve_options: Option<Resolve>,
    lazy_visit_modules: std::collections::HashSet<String>,
    issuer: Option<String>,
  ) {
    queue.add_task(FactorizeTask {
      original_module_identifier,
      issuer,
      original_resource_path,
      dependencies,
      is_entry,
      module_type,
      side_effects,
      resolve_options,
      lazy_visit_modules,
      options: self.compilation.options.clone(),
      plugin_driver: self.plugin_driver().clone(),
      cache: self.compilation.cache.clone(),
    });
  }
}
