use std::{path::PathBuf, sync::Arc};

use rspack_error::{Diagnostic, Result};

use crate::{
  cache::Cache, module_rule_matcher, BoxModuleDependency, BuildContext, BuildResult, Compilation,
  CompilerOptions, ContextModuleFactory, Dependency, DependencyType, LoaderRunnerRunner, Module,
  ModuleDependency, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleGraph,
  ModuleGraphModule, ModuleIdentifier, ModuleRule, ModuleType, NormalModuleFactory,
  NormalModuleFactoryContext, Resolve, SharedPluginDriver, WorkerQueue,
};

#[derive(Debug)]
pub enum TaskResult {
  Factorize(FactorizeTaskResult),
  Add(AddTaskResult),
  Build(BuildTaskResult),
  ProcessDependencies(ProcessDependenciesResult),
}

#[async_trait::async_trait]
pub trait WorkerTask {
  async fn run(self) -> Result<TaskResult>;
}

pub struct FactorizeTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_resource_path: Option<PathBuf>,
  pub dependencies: Vec<BoxModuleDependency>,

  pub is_entry: bool,
  pub module_type: Option<ModuleType>,
  pub side_effects: Option<bool>,
  pub resolve_options: Option<Resolve>,
  pub options: Arc<CompilerOptions>,
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
}

#[derive(Debug)]
pub struct FactorizeTaskResult {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub factory_result: ModuleFactoryResult,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxModuleDependency>,
  pub diagnostics: Vec<Diagnostic>,
  pub is_entry: bool,
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeTask {
  async fn run(self) -> Result<TaskResult> {
    let dependency = self.dependencies[0].clone();

    let (result, diagnostics) = match *dependency.dependency_type() {
      DependencyType::ImportContext => {
        let factory = ContextModuleFactory::new(self.plugin_driver, self.cache);
        factory
          .create(ModuleFactoryCreateData {
            resolve_options: self.resolve_options,
            context: None,
            dependency,
          })
          .await?
          .split_into_parts()
      }
      _ => {
        let factory = NormalModuleFactory::new(
          NormalModuleFactoryContext {
            original_resource_path: self.original_resource_path,
            module_type: self.module_type,
            side_effects: self.side_effects,
            options: self.options.clone(),
            lazy_visit_modules: self.lazy_visit_modules,
          },
          self.plugin_driver,
          self.cache,
        );
        factory
          .create(ModuleFactoryCreateData {
            resolve_options: self.resolve_options,
            context: dependency.get_context().map(|x| x.to_string()),
            dependency,
          })
          .await?
          .split_into_parts()
      }
    };

    let mut mgm = ModuleGraphModule::new(
      result.module.identifier(),
      *result.module.module_type(),
      // 1. if `tree_shaking` is false, then whatever `side_effects` is, all the module should be used by default.
      // 2. if `tree_shaking` is true, then only `side_effects` is false, `module.used` should be true.
      !self.options.builtins.tree_shaking || !self.options.builtins.side_effects,
    );

    mgm.set_issuer_if_unset(self.original_module_identifier);

    Ok(TaskResult::Factorize(FactorizeTaskResult {
      is_entry: self.is_entry,
      original_module_identifier: self.original_module_identifier,
      factory_result: result,
      module_graph_module: Box::new(mgm),
      dependencies: self.dependencies,
      diagnostics,
    }))
  }
}

pub type FactorizeQueue = WorkerQueue<FactorizeTask>;

pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: Box<dyn Module>,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxModuleDependency>,
  pub is_entry: bool,
}

#[derive(Debug)]
pub enum AddTaskResult {
  ModuleReused {
    module: Box<dyn Module>,
    dependencies: Vec<BoxModuleDependency>,
  },
  ModuleAdded {
    module: Box<dyn Module>,
    dependencies: Vec<BoxModuleDependency>,
  },
}

impl AddTask {
  pub fn run(self, compilation: &mut Compilation) -> Result<TaskResult> {
    let module_identifier = self.module.identifier();

    // TODO: Temporary module id, see TODOs of [VisitedModuleId]
    let temporary_module_id = (
      module_identifier,
      *self.dependencies[0].category(),
      self.dependencies[0].request().to_owned(),
    );

    if compilation.visited_module_id.contains(&temporary_module_id) {
      Self::set_resolved_module(
        &mut compilation.module_graph,
        self.original_module_identifier,
        self.dependencies.clone(),
        module_identifier,
      )?;

      return Ok(TaskResult::Add(AddTaskResult::ModuleReused {
        module: self.module,
        dependencies: self.dependencies,
      }));
    }

    compilation.visited_module_id.insert(temporary_module_id);

    compilation
      .module_graph
      .add_module_graph_module(*self.module_graph_module);

    Self::set_resolved_module(
      &mut compilation.module_graph,
      self.original_module_identifier,
      self.dependencies.clone(),
      module_identifier,
    )?;

    if self.is_entry {
      compilation
        .entry_module_identifiers
        .insert(module_identifier);
    }

    Ok(TaskResult::Add(AddTaskResult::ModuleAdded {
      module: self.module,
      dependencies: self.dependencies,
    }))
  }
}

impl AddTask {
  fn set_resolved_module(
    module_graph: &mut ModuleGraph,
    original_module_identifier: Option<ModuleIdentifier>,
    dependencies: Vec<BoxModuleDependency>,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    for dependency in dependencies {
      let dep_id = module_graph.add_dependency(dependency, module_identifier);
      module_graph.set_resolved_module(original_module_identifier, dep_id, module_identifier)?;
    }

    Ok(())
  }
}

pub type AddQueue = WorkerQueue<AddTask>;

pub struct BuildTask {
  pub module: Box<dyn Module>,
  pub dependencies: Vec<BoxModuleDependency>,

  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
}

#[derive(Debug)]
pub struct BuildTaskResult {
  pub module: Box<dyn Module>,
  pub build_result: Box<BuildResult>,
  pub dependencies: Vec<BoxModuleDependency>,
  pub diagnostics: Vec<Diagnostic>,
}

#[async_trait::async_trait]
impl WorkerTask for BuildTask {
  async fn run(self) -> Result<TaskResult> {
    let mut module = self.module;
    let compiler_options = self.compiler_options;
    let loader_runner_runner = self.loader_runner_runner;
    let cache = self.cache;
    let plugin_driver = self.plugin_driver;

    let build_result = cache
      .build_module_occasion
      .use_cache(&mut module, |module| async {
        let resolved_loaders = if let Some(normal_module) = module.as_normal_module() {
          let resource_data = normal_module.resource_resolved_data();

          compiler_options
            .module
            .rules
            .iter()
            .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
              match module_rule_matcher(module_rule, resource_data) {
                Ok(val) => val.then_some(Ok(module_rule)),
                Err(err) => Some(Err(err)),
              }
            })
            .collect::<Result<Vec<_>>>()?
        } else {
          vec![]
        };

        let resolved_loaders = resolved_loaders
          .into_iter()
          .flat_map(|module_rule| module_rule.r#use.iter().map(Box::as_ref).rev())
          .collect::<Vec<_>>();

        plugin_driver
          .read()
          .await
          .build_module(module.as_mut())
          .await?;

        let result = module
          .build(BuildContext {
            resolved_loaders,
            loader_runner_runner: &loader_runner_runner,
            compiler_options: &compiler_options,
          })
          .await;

        plugin_driver.read().await.succeed_module(module).await?;

        result
      })
      .await;

    build_result.map(|build_result| {
      let (build_result, diagnostics) = build_result.split_into_parts();
      TaskResult::Build(BuildTaskResult {
        module,
        dependencies: self.dependencies,
        build_result: Box::new(build_result),
        diagnostics,
      })
    })
  }
}

pub type BuildQueue = WorkerQueue<BuildTask>;

pub struct ProcessDependenciesTask {
  pub original_module_identifier: ModuleIdentifier,
  pub dependencies: Vec<BoxModuleDependency>,
  pub resolve_options: Option<Resolve>,
}

#[derive(Debug)]
pub struct ProcessDependenciesResult {
  pub module_identifier: ModuleIdentifier,
}

pub type ProcessDependenciesQueue = WorkerQueue<ProcessDependenciesTask>;

pub struct CleanTask {
  pub module_identifier: ModuleIdentifier,
}

#[derive(Debug)]
pub enum CleanTaskResult {
  ModuleIsUsed {
    module_identifier: ModuleIdentifier,
  },
  ModuleIsCleaned {
    module_identifier: ModuleIdentifier,
    dependent_module_identifiers: Vec<ModuleIdentifier>,
  },
}

impl CleanTask {
  pub fn run(self, compilation: &mut Compilation) -> CleanTaskResult {
    let module_identifier = self.module_identifier;
    let mgm = match compilation
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
    {
      Some(mgm) => mgm,
      None => {
        return CleanTaskResult::ModuleIsCleaned {
          module_identifier,
          dependent_module_identifiers: vec![],
        }
      }
    };

    if !mgm.incoming_connections.is_empty() {
      return CleanTaskResult::ModuleIsUsed { module_identifier };
    }

    let dependent_module_identifiers: Vec<ModuleIdentifier> = mgm
      .all_depended_modules(&compilation.module_graph)
      .iter()
      .map(|mgm| mgm.module_identifier)
      .collect();
    compilation.module_graph.revoke_module(&module_identifier);
    CleanTaskResult::ModuleIsCleaned {
      module_identifier,
      dependent_module_identifiers,
    }
  }
}

pub type CleanQueue = WorkerQueue<CleanTask>;
