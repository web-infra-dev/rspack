mod queue;
mod rebuild_deps_builder;

use std::{
  collections::VecDeque,
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use rayon::prelude::*;
use rspack_error::Result;
use rspack_identifier::Identifier;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

pub use self::queue::FactorizeTask;
use self::queue::{
  AddQueue, AddTask, AddTaskResult, BuildQueue, BuildTask, BuildTaskResult, CleanQueue, CleanTask,
  CleanTaskResult, FactorizeQueue, FactorizeTaskResult, ProcessDependenciesQueue,
  ProcessDependenciesResult, ProcessDependenciesTask, TaskResult, WorkerTask,
};
pub use self::rebuild_deps_builder::RebuildDepsBuilder;
use crate::{
  logger::Logger, tree_shaking::BailoutFlag, AsyncDependenciesBlock,
  AsyncDependenciesBlockIdentifier, BoxDependency, BuildDependency, CacheOptions, Compilation,
  Context, ContextDependency, DependencyId, DependencyParents, DependencyType, GroupOptions,
  Module, ModuleFactoryResult, ModuleGraph, ModuleGraphModule, ModuleIdentifier, ModuleIssuer,
  ModuleProfile, NormalModuleSource, Resolve,
};

#[derive(Debug)]
pub enum MakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  DeletedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

impl MakeParam {
  pub fn new_force_build_dep_param(dep: DependencyId, module: Option<ModuleIdentifier>) -> Self {
    let mut data = HashSet::default();
    data.insert((dep, module));
    Self::ForceBuildDeps(data)
  }
}

pub async fn update_module_graph(
  compilation: &mut Compilation,
  params: Vec<MakeParam>,
) -> Result<()> {
  let mut builder = UpdateModuleGraph::default();
  let build_dependencies = builder.cutout(compilation, params)?;
  builder.repair(compilation, build_dependencies)
}

type ModuleDeps = (
  Vec<Identifier>,
  Vec<(AsyncDependenciesBlockIdentifier, Option<GroupOptions>)>,
);

struct UpdateModuleGraph {
  origin_module_deps: HashMap<Identifier, ModuleDeps>,
  /// Rebuild module issuer mappings
  origin_module_issuers: HashMap<Identifier, ModuleIssuer>,

  factorize_queue: FactorizeQueue,
  add_queue: AddQueue,
  build_queue: BuildQueue,
  process_dependencies_queue: ProcessDependenciesQueue,

  make_failed_dependencies: HashSet<BuildDependency>,
  make_failed_module: HashSet<ModuleIdentifier>,

  need_check_isolated_module_ids: HashSet<Identifier>,

  active_task_count: usize,
  is_expected_shutdown: Arc<AtomicBool>,

  result_tx: mpsc::UnboundedSender<Result<TaskResult>>,
  result_rx: mpsc::UnboundedReceiver<Result<TaskResult>>,
}

impl Default for UpdateModuleGraph {
  fn default() -> Self {
    let (result_tx, result_rx) = mpsc::unbounded_channel::<Result<TaskResult>>();
    Self {
      origin_module_deps: HashMap::default(),
      origin_module_issuers: HashMap::default(),
      factorize_queue: FactorizeQueue::new(),
      add_queue: AddQueue::new(),
      build_queue: BuildQueue::new(),
      process_dependencies_queue: ProcessDependenciesQueue::new(),
      make_failed_dependencies: HashSet::default(),
      make_failed_module: HashSet::default(),
      need_check_isolated_module_ids: HashSet::default(),
      active_task_count: 0,
      is_expected_shutdown: Arc::new(AtomicBool::new(false)),
      result_tx,
      result_rx,
    }
  }
}

impl UpdateModuleGraph {
  fn cutout(
    &mut self,
    compilation: &mut Compilation,
    params: Vec<MakeParam>,
  ) -> Result<HashSet<BuildDependency>> {
    let deps_builder = RebuildDepsBuilder::new(params, &compilation.get_module_graph());

    self.origin_module_deps = HashMap::from_iter(
      deps_builder
        .get_force_build_modules()
        .iter()
        .map(|module_identifier| {
          (
            *module_identifier,
            Self::module_deps(compilation, module_identifier),
          )
        }),
    );

    let module_graph = compilation.get_module_graph();
    // calc need_check_isolated_module_ids & regen_module_issues
    for id in deps_builder.get_force_build_modules() {
      if let Some(mgm) = compilation
        .get_module_graph()
        .module_graph_module_by_identifier(id)
      {
        let depended_modules = module_graph
          .get_module_all_depended_modules(id)
          .expect("module graph module not exist")
          .into_iter()
          .copied();
        self.need_check_isolated_module_ids.extend(depended_modules);
        self
          .origin_module_issuers
          .insert(*id, mgm.get_issuer().clone());
      }
    }

    Ok(deps_builder.revoke_modules(&mut compilation.get_module_graph_mut()))
  }

  fn repair(
    &mut self,
    compilation: &mut Compilation,
    build_dependencies: HashSet<BuildDependency>,
  ) -> Result<()> {
    let logger = compilation.get_logger("rspack.Compilation");

    let mut errored = None;

    let module_graph = compilation.get_module_graph();
    build_dependencies
      .into_iter()
      .for_each(|(id, parent_module_identifier)| {
        let dependency = module_graph
          .dependency_by_id(&id)
          .expect("dependency not found");
        if dependency.as_module_dependency().is_none()
          && dependency.as_context_dependency().is_none()
        {
          return;
        }

        let parent_module =
          parent_module_identifier.and_then(|id| module_graph.module_by_identifier(&id));
        if parent_module_identifier.is_some() && parent_module.is_none() {
          return;
        }
        self.handle_module_creation(
          compilation,
          parent_module_identifier,
          parent_module.and_then(|m| m.get_context()),
          vec![id],
          parent_module_identifier.is_none(),
          parent_module.and_then(|module| module.get_resolve_options()),
          compilation.lazy_visit_modules.clone(),
          parent_module
            .and_then(|m| m.as_normal_module())
            .and_then(|module| module.name_for_condition()),
          true,
        );
      });

    let mut add_time = logger.time_aggregate("module add task");
    let mut process_deps_time = logger.time_aggregate("module process dependencies task");
    let mut factorize_time = logger.time_aggregate("module factorize task");
    let mut build_time = logger.time_aggregate("module build task");

    let mut build_cache_counter = None;
    let mut factorize_cache_counter = None;

    if !(matches!(compilation.options.cache, CacheOptions::Disabled)) {
      build_cache_counter = Some(logger.cache("module build cache"));
      factorize_cache_counter = Some(logger.cache("module factorize cache"));
    }

    tokio::task::block_in_place(|| loop {
      let start = factorize_time.start();

      while let Some(task) = self.factorize_queue.get_task(compilation) {
        self.active_task_count += 1;

        // TODO: change when we insert dependency to module_graph
        compilation
          .get_module_graph_mut()
          .add_dependency(task.dependency.clone());

        tokio::spawn({
          let result_tx = self.result_tx.clone();
          let is_expected_shutdown = self.is_expected_shutdown.clone();

          async move {
            if is_expected_shutdown.load(Ordering::Relaxed) {
              return;
            }

            let result = task.run().await;
            if !is_expected_shutdown.load(Ordering::Relaxed) {
              result_tx
                .send(result)
                .expect("Failed to send factorize result");
            }
          }
        });
      }
      factorize_time.end(start);

      let start = build_time.start();
      while let Some(task) = self.build_queue.get_task(compilation) {
        self.active_task_count += 1;
        tokio::spawn({
          let result_tx = self.result_tx.clone();
          let is_expected_shutdown = self.is_expected_shutdown.clone();

          async move {
            if is_expected_shutdown.load(Ordering::Relaxed) {
              return;
            }

            let result = task.run().await;
            if !is_expected_shutdown.load(Ordering::Relaxed) {
              result_tx.send(result).expect("Failed to send build result");
            }
          }
        });
      }
      build_time.end(start);

      let start = add_time.start();
      while let Some(task) = self.add_queue.get_task(compilation) {
        self.active_task_count += 1;
        let result = task.run(compilation);
        self
          .result_tx
          .send(result)
          .expect("Failed to send add result");
      }
      add_time.end(start);

      let start = process_deps_time.start();
      while let Some(task) = self.process_dependencies_queue.get_task(compilation) {
        self.active_task_count += 1;

        let mut sorted_dependencies = HashMap::default();
        let module_graph = compilation.get_module_graph();

        task.dependencies.into_iter().for_each(|dependency_id| {
          let dependency = module_graph
            .dependency_by_id(&dependency_id)
            .expect("should have dependency");
          // FIXME: now only module/context dependency can put into resolve queue.
          // FIXME: should align webpack
          let resource_identifier =
            if let Some(module_dependency) = dependency.as_module_dependency() {
              // TODO need implement more dependency `resource_identifier()`
              // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L1621
              let id = if let Some(resource_identifier) = module_dependency.resource_identifier() {
                resource_identifier.to_string()
              } else {
                format!(
                  "{}|{}",
                  module_dependency.dependency_type(),
                  module_dependency.request()
                )
              };
              Some(id)
            } else {
              dependency
                .as_context_dependency()
                .map(|d| ContextDependency::resource_identifier(d).to_string())
            };

          if let Some(resource_identifier) = resource_identifier {
            sorted_dependencies
              .entry(resource_identifier)
              .or_insert(vec![])
              .push(dependency_id);
          }
        });

        let original_module_identifier = &task.original_module_identifier;
        let module = module_graph
          .module_by_identifier(original_module_identifier)
          .expect("Module expected");

        for dependencies in sorted_dependencies.into_values() {
          self.handle_module_creation(
            compilation,
            Some(module.identifier()),
            module.get_context(),
            dependencies,
            false,
            task.resolve_options.clone(),
            compilation.lazy_visit_modules.clone(),
            module
              .as_normal_module()
              .and_then(|module| module.name_for_condition()),
            true,
          );
        }

        tokio::spawn({
          let tx = self.result_tx.clone();
          let is_expected_shutdown = self.is_expected_shutdown.clone();
          async move {
            if is_expected_shutdown.load(Ordering::Relaxed) {
              return;
            }

            tx.send(Ok(TaskResult::ProcessDependencies(Box::new(
              ProcessDependenciesResult {
                module_identifier: task.original_module_identifier,
              },
            ))))
            .expect("Failed to send process dependencies result");
          }
        });
      }
      process_deps_time.end(start);

      match self.result_rx.try_recv() {
        Ok(item) => {
          if let Ok(item) = &item {
            match item {
              TaskResult::Factorize(result) => {
                if let Some(ModuleFactoryResult {
                  module: Some(module),
                  ..
                }) = &result.factory_result
                {
                  self.factorize_queue.complete_task(
                    result.dependency,
                    module.identifier(),
                    compilation,
                  )
                }
              }
              TaskResult::Add(result) => {
                let module = match result.as_ref() {
                  AddTaskResult::ModuleReused { module } => module.identifier(),
                  AddTaskResult::ModuleAdded { module, .. } => module.identifier(),
                };

                self.add_queue.complete_task(module, module, compilation)
              }
              TaskResult::Build(result) => {
                let id = result.module.identifier();
                self.build_queue.complete_task(id, id, compilation);
              }
              TaskResult::ProcessDependencies(result) => {
                self.process_dependencies_queue.complete_task(
                  result.module_identifier,
                  result.module_identifier,
                  compilation,
                );
              }
            }
          }

          match item {
            Ok(TaskResult::Factorize(box task_result)) => {
              let FactorizeTaskResult {
                original_module_identifier,
                factory_result,
                dependencies,
                is_entry,
                current_profile,
                exports_info_related,
                file_dependencies,
                context_dependencies,
                missing_dependencies,
                diagnostics,
                connect_origin,
                ..
              } = task_result;
              if !diagnostics.is_empty() {
                if let Some(id) = original_module_identifier {
                  self.make_failed_module.insert(id);
                } else {
                  self
                    .make_failed_dependencies
                    .insert((dependencies[0], None));
                }
              }

              compilation.push_batch_diagnostic(
                diagnostics
                  .into_iter()
                  .map(|d| d.with_module_identifier(original_module_identifier))
                  .collect(),
              );

              compilation.file_dependencies.extend(file_dependencies);
              compilation
                .context_dependencies
                .extend(context_dependencies);
              compilation
                .missing_dependencies
                .extend(missing_dependencies);

              if let Some(factory_result) = factory_result {
                if let Some(counter) = &mut factorize_cache_counter {
                  if factory_result.from_cache {
                    counter.hit();
                  } else {
                    counter.miss();
                  }
                }

                if let Some(module) = factory_result.module {
                  let module_identifier = module.identifier();
                  let mut mgm = ModuleGraphModule::new(
                    module.identifier(),
                    *module.module_type(),
                    exports_info_related.exports_info.id,
                  );
                  mgm.set_issuer_if_unset(original_module_identifier);

                  let mut module_graph = compilation.get_module_graph_mut();
                  module_graph.set_exports_info(
                    exports_info_related.exports_info.id,
                    exports_info_related.exports_info,
                  );
                  module_graph.set_export_info(
                    exports_info_related.side_effects_info.id,
                    exports_info_related.side_effects_info,
                  );
                  module_graph.set_export_info(
                    exports_info_related.other_exports_info.id,
                    exports_info_related.other_exports_info,
                  );

                  self.add_queue.add_task(AddTask {
                    original_module_identifier,
                    module,
                    module_graph_module: Box::new(mgm),
                    dependencies,
                    is_entry,
                    current_profile,
                    connect_origin,
                  });
                  tracing::trace!("Module created: {}", &module_identifier);
                } else {
                  let module_graph = compilation.get_module_graph();
                  let dep = module_graph
                    .dependency_by_id(&dependencies[0])
                    .expect("dep should available");
                  tracing::trace!("Module ignored: {dep:?}")
                }
              } else {
                let module_graph = compilation.get_module_graph();
                let dep = module_graph
                  .dependency_by_id(&dependencies[0])
                  .expect("dep should available");
                tracing::trace!("Module created with failure, but without bailout: {dep:?}");
              }
            }
            Ok(TaskResult::Add(box task_result)) => match task_result {
              AddTaskResult::ModuleAdded {
                module,
                current_profile,
              } => {
                tracing::trace!("Module added: {}", module.identifier());
                self.build_queue.add_task(BuildTask {
                  module,
                  resolver_factory: compilation.resolver_factory.clone(),
                  compiler_options: compilation.options.clone(),
                  plugin_driver: compilation.plugin_driver.clone(),
                  cache: compilation.cache.clone(),
                  current_profile,
                });
              }
              AddTaskResult::ModuleReused { module, .. } => {
                tracing::trace!("Module reused: {}, skipping build", module.identifier());

                let module_identifier = module.identifier();
                if compilation
                  .get_module_graph()
                  .module_by_identifier(&module_identifier)
                  .is_some()
                {
                  self.active_task_count += 1;
                  self
                    .result_tx
                    .send(Ok(TaskResult::ProcessDependencies(Box::new(
                      ProcessDependenciesResult {
                        module_identifier: module.identifier(),
                      },
                    ))))
                    .expect("Failed to send factorize result");
                }
              }
            },
            Ok(TaskResult::Build(box task_result)) => {
              let BuildTaskResult {
                mut module,
                build_result,
                diagnostics,
                current_profile,
                from_cache,
              } = task_result;

              if let Some(counter) = &mut build_cache_counter {
                if from_cache {
                  counter.hit();
                } else {
                  counter.miss();
                }
              }

              if compilation.options.builtins.tree_shaking.enable() {
                compilation
                  .optimize_analyze_result_map
                  .insert(module.identifier(), build_result.analyze_result);
              }

              if !diagnostics.is_empty() {
                self.make_failed_module.insert(module.identifier());
              }

              tracing::trace!("Module built: {}", module.identifier());
              compilation.push_batch_diagnostic(diagnostics);
              compilation
                .get_module_graph_mut()
                .get_optimization_bailout_mut(&module.identifier())
                .extend(build_result.optimization_bailouts);
              compilation
                .file_dependencies
                .extend(build_result.build_info.file_dependencies.clone());
              compilation
                .context_dependencies
                .extend(build_result.build_info.context_dependencies.clone());
              compilation
                .missing_dependencies
                .extend(build_result.build_info.missing_dependencies.clone());
              compilation
                .build_dependencies
                .extend(build_result.build_info.build_dependencies.clone());

              let mut queue = VecDeque::new();
              let mut all_dependencies = vec![];
              let mut handle_block =
                |dependencies: Vec<BoxDependency>,
                 blocks: Vec<AsyncDependenciesBlock>,
                 queue: &mut VecDeque<AsyncDependenciesBlock>,
                 module_graph: &mut ModuleGraph,
                 current_block: Option<AsyncDependenciesBlock>| {
                  for dependency in dependencies {
                    let dependency_id = *dependency.id();
                    if current_block.is_none() {
                      module.add_dependency_id(dependency_id);
                    }
                    all_dependencies.push(dependency_id);
                    module_graph.set_parents(
                      dependency_id,
                      DependencyParents {
                        block: current_block.as_ref().map(|block| block.identifier()),
                        module: module.identifier(),
                      },
                    );
                    module_graph.add_dependency(dependency);
                  }
                  if let Some(current_block) = current_block {
                    module.add_block_id(current_block.identifier());
                    module_graph.add_block(current_block);
                  }
                  for block in blocks {
                    queue.push_back(block);
                  }
                };
              handle_block(
                build_result.dependencies,
                build_result.blocks,
                &mut queue,
                &mut compilation.get_module_graph_mut(),
                None,
              );
              while let Some(mut block) = queue.pop_front() {
                let dependencies = block.take_dependencies();
                let blocks = block.take_blocks();
                handle_block(
                  dependencies,
                  blocks,
                  &mut queue,
                  &mut compilation.get_module_graph_mut(),
                  Some(block),
                );
              }

              {
                let mut module_graph = compilation.get_module_graph_mut();
                let mgm = module_graph
                  .module_graph_module_by_identifier_mut(&module.identifier())
                  .expect("Failed to get mgm");
                mgm.__deprecated_all_dependencies = all_dependencies.clone();
                if let Some(current_profile) = current_profile {
                  mgm.set_profile(current_profile);
                }
              }

              let module_identifier = module.identifier();

              module.set_build_info(build_result.build_info);
              module.set_build_meta(build_result.build_meta);

              let mut mg = compilation.get_module_graph_mut();

              let resolve_options = module.get_resolve_options();
              mg.add_module(module);

              self
                .process_dependencies_queue
                .add_task(ProcessDependenciesTask {
                  dependencies: all_dependencies,
                  original_module_identifier: module_identifier,
                  resolve_options,
                });
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
              self.is_expected_shutdown.store(true, Ordering::Relaxed);
              break;
            }
          }

          self.active_task_count -= 1;
        }
        Err(TryRecvError::Disconnected) => {
          self.is_expected_shutdown.store(true, Ordering::Relaxed);
          break;
        }
        Err(TryRecvError::Empty) => {
          if self.active_task_count == 0 {
            self.is_expected_shutdown.store(true, Ordering::Relaxed);
            break;
          }
        }
      }
    });
    logger.time_aggregate_end(add_time);
    logger.time_aggregate_end(process_deps_time);
    logger.time_aggregate_end(factorize_time);
    logger.time_aggregate_end(build_time);

    if let Some(counter) = build_cache_counter {
      logger.cache_end(counter);
    }
    if let Some(counter) = factorize_cache_counter {
      logger.cache_end(counter);
    }

    compilation
      .make_failed_dependencies
      .extend(self.make_failed_dependencies.drain());
    compilation
      .make_failed_module
      .extend(self.make_failed_module.drain());
    tracing::debug!("All task is finished");

    // clean isolated module
    let mut clean_queue = CleanQueue::new();
    clean_queue.add_tasks(
      self
        .need_check_isolated_module_ids
        .drain()
        .map(|module_identifier| CleanTask { module_identifier }),
    );

    while let Some(task) = clean_queue.get_task(compilation) {
      match task.run(compilation) {
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
    for (id, issuer) in self.origin_module_issuers.drain() {
      if let Some(mgm) = compilation
        .get_module_graph_mut()
        .module_graph_module_by_identifier_mut(&id)
      {
        mgm.set_issuer(issuer);
      }
    }

    // calc has_module_import_export_change
    compilation.has_module_import_export_change = if self.origin_module_deps.is_empty() {
      true
    } else {
      compilation.has_module_import_export_change
        || !self.origin_module_deps.drain().all(|(module_id, deps)| {
          if compilation
            .get_module_graph_mut()
            .module_by_identifier(&module_id)
            .is_none()
          {
            false
          } else {
            let (now_deps, mut now_blocks) = Self::module_deps(compilation, &module_id);
            let (origin_deps, mut origin_blocks) = deps;
            if now_deps.len() != origin_deps.len() || now_blocks.len() != origin_blocks.len() {
              false
            } else {
              for index in 0..origin_deps.len() {
                if origin_deps[index] != now_deps[index] {
                  return false;
                }
              }

              now_blocks.sort_unstable();
              origin_blocks.sort_unstable();

              for index in 0..origin_blocks.len() {
                if origin_blocks[index].0 != now_blocks[index].0 {
                  return false;
                }
                if origin_blocks[index].1 != now_blocks[index].1 {
                  return false;
                }
              }

              true
            }
          }
        })
    };

    // Avoid to introduce too much overhead,
    // until we find a better way to align with webpack hmr behavior

    // add context module and context element module to bailout_module_identifiers
    if compilation.options.builtins.tree_shaking.enable() {
      compilation.bailout_module_identifiers = compilation
        .get_module_graph()
        .dependencies()
        .values()
        .par_bridge()
        .filter_map(|dep| {
          if dep.as_context_dependency().is_some()
            && let Some(module) = compilation
              .get_module_graph()
              .get_module_by_dependency_id(dep.id())
          {
            let mut values = vec![(module.identifier(), BailoutFlag::CONTEXT_MODULE)];
            if let Some(dependencies) = compilation
              .get_module_graph()
              .get_module_all_dependencies(&module.identifier())
            {
              for dependency in dependencies {
                if let Some(dependency_module) = compilation
                  .get_module_graph()
                  .module_identifier_by_dependency_id(dependency)
                {
                  values.push((*dependency_module, BailoutFlag::CONTEXT_MODULE));
                }
              }
            }

            Some(values)
          } else if matches!(
            dep.dependency_type(),
            DependencyType::ContainerExposed | DependencyType::ProvideModuleForShared
          ) && let Some(module) = compilation
            .get_module_graph()
            .get_module_by_dependency_id(dep.id())
          {
            Some(vec![(module.identifier(), BailoutFlag::CONTAINER_EXPOSED)])
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
    &mut self,
    compilation: &Compilation,
    original_module_identifier: Option<ModuleIdentifier>,
    original_module_context: Option<Box<Context>>,
    dependencies: Vec<DependencyId>,
    is_entry: bool,
    resolve_options: Option<Box<Resolve>>,
    lazy_visit_modules: std::collections::HashSet<String>,
    issuer: Option<Box<str>>,
    connect_origin: bool,
  ) {
    let current_profile = compilation
      .options
      .profile
      .then(Box::<ModuleProfile>::default);
    let dependency = compilation
      .get_module_graph()
      .dependency_by_id(&dependencies[0])
      .expect("should have dependency")
      .clone();
    let module_graph = compilation.get_module_graph();
    let original_module_source = original_module_identifier
      .and_then(|i| module_graph.module_by_identifier(&i))
      .and_then(|m| m.as_normal_module())
      .and_then(|m| {
        if let NormalModuleSource::BuiltSucceed(s) = m.source() {
          Some(s.clone())
        } else {
          None
        }
      });
    self.factorize_queue.add_task(FactorizeTask {
      module_factory: compilation.get_dependency_factory(&dependency),
      original_module_identifier,
      original_module_source,
      issuer,
      original_module_context,
      dependency,
      dependencies,
      is_entry,
      resolve_options,
      lazy_visit_modules,
      resolver_factory: compilation.resolver_factory.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      options: compilation.options.clone(),
      plugin_driver: compilation.plugin_driver.clone(),
      cache: compilation.cache.clone(),
      current_profile,
      connect_origin,
    });
  }

  fn module_deps(compilation: &Compilation, module_identifier: &ModuleIdentifier) -> ModuleDeps {
    let module_graph = compilation.get_module_graph();
    let (deps, blocks) = module_graph.get_module_dependencies_modules_and_blocks(module_identifier);

    let blocks_with_option: Vec<_> = blocks
      .iter()
      .map(|block| {
        (
          *block,
          compilation
            .get_module_graph()
            .block_by_id(block)
            .expect("block muse be exist")
            .get_group_options()
            .cloned(),
        )
      })
      .collect();
    (deps, blocks_with_option)
  }
}
