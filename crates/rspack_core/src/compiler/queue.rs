use std::sync::Arc;

use rspack_error::{internal_error, Diagnostic, Error, Result};

use crate::{
  cache::Cache, module_rule_matcher, BuildContext, BuildResult, Compilation, CompilerOptions,
  Dependency, FactorizeResult, LoaderRunnerRunner, Module, ModuleDependency, ModuleGraph,
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
  pub dependencies: Vec<Box<dyn ModuleDependency>>,

  pub is_entry: bool,
  pub module_name: Option<String>,
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
  pub factory_result: FactorizeResult,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  pub is_entry: bool,
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeTask {
  async fn run(self) -> Result<TaskResult> {
    let factory = NormalModuleFactory::new(
      NormalModuleFactoryContext {
        module_name: self.module_name,
        module_type: self.module_type,
        side_effects: self.side_effects,
        options: self.options,
        lazy_visit_modules: self.lazy_visit_modules,
      },
      self.dependencies[0].clone(),
      self.plugin_driver,
      self.cache,
    );

    let (result, context) = factory.create(self.resolve_options).await?;
    let mut mgm = ModuleGraphModule::new(
      context.module_name.clone(),
      result.module.identifier(),
      context.module_type.ok_or_else(|| {
        Error::InternalError(internal_error!(format!(
          "Unable to get the module type for module {}, did you forget to configure `Rule.type`? ",
          result.module.identifier()
        )))
      })?,
      !context.options.builtins.side_effects,
    );

    mgm.set_issuer_if_unset(self.original_module_identifier);

    Ok(TaskResult::Factorize(FactorizeTaskResult {
      is_entry: self.is_entry,
      original_module_identifier: self.original_module_identifier,
      factory_result: result,
      module_graph_module: Box::new(mgm),
      dependencies: self.dependencies,
    }))
  }
}

pub type FactorizeQueue = WorkerQueue<FactorizeTask>;

pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: Box<dyn Module>,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  pub is_entry: bool,
}

#[derive(Debug)]
pub enum AddTaskResult {
  ModuleReused(Box<dyn Module>),
  ModuleAdded(Box<dyn Module>),
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
        self.dependencies,
        module_identifier,
      )?;

      return Ok(TaskResult::Add(AddTaskResult::ModuleReused(self.module)));
    }

    compilation.visited_module_id.insert(temporary_module_id);

    compilation
      .module_graph
      .add_module_graph_module(*self.module_graph_module);

    Self::set_resolved_module(
      &mut compilation.module_graph,
      self.original_module_identifier,
      self.dependencies,
      module_identifier,
    )?;

    if self.is_entry {
      compilation
        .entry_module_identifiers
        .insert(module_identifier);
    }

    Ok(TaskResult::Add(AddTaskResult::ModuleAdded(self.module)))
  }
}

impl AddTask {
  fn set_resolved_module(
    module_graph: &mut ModuleGraph,
    original_module_identifier: Option<ModuleIdentifier>,
    dependencies: Vec<Box<dyn ModuleDependency>>,
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

  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
}

#[derive(Debug)]
pub enum BuildTaskResult {
  BuildSuccess {
    module: Box<dyn Module>,
    build_result: Box<BuildResult>,
    diagnostics: Vec<Diagnostic>,
  },
  BuildWithError {
    module: Box<dyn Module>,
    diagnostics: Vec<Diagnostic>,
  },
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

    match build_result {
      Ok(build_result) => {
        let (build_result, diagnostics) = build_result.split_into_parts();
        Ok(TaskResult::Build(BuildTaskResult::BuildSuccess {
          module,
          build_result: Box::new(build_result),
          diagnostics,
        }))
      }
      Err(err) => Ok(TaskResult::Build(BuildTaskResult::BuildWithError {
        module,
        diagnostics: err.into(),
      })),
    }
  }
}

pub type BuildQueue = WorkerQueue<BuildTask>;

pub struct ProcessDependenciesTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  pub resolve_options: Option<Resolve>,
}

#[derive(Debug)]
pub struct ProcessDependenciesResult {
  pub module_identifier: ModuleIdentifier,
}

pub type ProcessDependenciesQueue = WorkerQueue<ProcessDependenciesTask>;
