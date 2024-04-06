use std::path::PathBuf;
use std::sync::Arc;

use derivative::Derivative;
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result};
use rspack_sources::BoxSource;
use rustc_hash::FxHashSet as HashSet;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  cache::Cache, BoxDependency, BuildContext, BuildResult, Compilation, CompilerContext,
  CompilerOptions, Context, Module, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult,
  ModuleGraph, ModuleGraphModule, ModuleIdentifier, ModuleProfile, Resolve, ResolverFactory,
  SharedPluginDriver, WorkerQueue,
};
use crate::{
  BoxModule, DependencyId, ExecuteModuleResult, ExportInfo, ExportsInfo, QueueHandler, UsageState,
};

pub type CleanQueue = WorkerQueue<CleanTask, ModuleIdentifier>;

pub type ModuleCreationCallback = Box<dyn FnOnce(&BoxModule) + Send>;

pub type FactorizeQueueHandler = QueueHandler<FactorizeTask, DependencyId>;
pub type AddQueueHandler = QueueHandler<AddTask, ModuleIdentifier>;
pub type BuildQueueHandler = QueueHandler<BuildTask, ModuleIdentifier>;
pub type ProcessDependenciesQueueHandler = QueueHandler<ProcessDependenciesTask, ModuleIdentifier>;
pub type BuildTimeExecutionQueueHandler = QueueHandler<BuildTimeExecutionTask, ModuleIdentifier>;

#[derive(Debug)]
pub enum TaskResult {
  Factorize(Box<FactorizeTaskResult>),
  Add(Box<AddTaskResult>),
  Build(Box<BuildTaskResult>),
  ProcessDependencies(Box<ProcessDependenciesResult>),
}

#[async_trait::async_trait]
pub trait WorkerTask {
  async fn run(self) -> Result<TaskResult>;
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FactorizeTask {
  pub module_factory: Arc<dyn ModuleFactory>,
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_source: Option<BoxSource>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub dependency: BoxDependency,
  pub dependencies: Vec<DependencyId>,
  pub is_entry: bool,
  pub resolve_options: Option<Box<Resolve>>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub options: Arc<CompilerOptions>,
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub connect_origin: bool,
  #[derivative(Debug = "ignore")]
  pub callback: Option<ModuleCreationCallback>,
}

/// a struct temporarily used creating ExportsInfo
#[derive(Debug)]
pub struct ExportsInfoRelated {
  pub exports_info: ExportsInfo,
  pub other_exports_info: ExportInfo,
  pub side_effects_info: ExportInfo,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FactorizeTaskResult {
  pub dependency: DependencyId,
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// Result will be available if [crate::ModuleFactory::create] returns `Ok`.
  pub factory_result: Option<ModuleFactoryResult>,
  pub dependencies: Vec<DependencyId>,
  pub is_entry: bool,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub exports_info_related: ExportsInfoRelated,

  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub diagnostics: Vec<Diagnostic>,
  #[derivative(Debug = "ignore")]
  pub callback: Option<ModuleCreationCallback>,
  pub connect_origin: bool,
}

impl FactorizeTaskResult {
  fn with_factory_result(mut self, factory_result: Option<ModuleFactoryResult>) -> Self {
    self.factory_result = factory_result;
    self
  }

  fn with_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
    self.diagnostics = diagnostics;
    self
  }

  fn with_file_dependencies(mut self, files: impl IntoIterator<Item = PathBuf>) -> Self {
    self.file_dependencies = files.into_iter().collect();
    self
  }

  fn with_context_dependencies(mut self, contexts: impl IntoIterator<Item = PathBuf>) -> Self {
    self.context_dependencies = contexts.into_iter().collect();
    self
  }

  fn with_missing_dependencies(mut self, missing: impl IntoIterator<Item = PathBuf>) -> Self {
    self.missing_dependencies = missing.into_iter().collect();
    self
  }
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeTask {
  async fn run(self) -> Result<TaskResult> {
    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_factory_start();
    }
    let dependency = self.dependency;
    let dep_id = *dependency.id();

    let context = if let Some(context) = dependency.get_context() {
      context
    } else if let Some(context) = &self.original_module_context {
      context
    } else {
      &self.options.context
    }
    .clone();

    let other_exports_info = ExportInfo::new(None, UsageState::Unknown, None);
    let side_effects_only_info = ExportInfo::new(
      Some("*side effects only*".into()),
      UsageState::Unknown,
      None,
    );
    let exports_info = ExportsInfo::new(other_exports_info.id, side_effects_only_info.id);
    let factorize_task_result = FactorizeTaskResult {
      dependency: dep_id,
      original_module_identifier: self.original_module_identifier,
      factory_result: None,
      dependencies: self.dependencies,
      is_entry: self.is_entry,
      current_profile: self.current_profile,
      exports_info_related: ExportsInfoRelated {
        exports_info,
        other_exports_info,
        side_effects_info: side_effects_only_info,
      },
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      diagnostics: Default::default(),
      connect_origin: self.connect_origin,
      callback: self.callback,
    };

    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      resolve_options: self.resolve_options,
      context,
      dependency,
      issuer: self.issuer,
      issuer_identifier: self.original_module_identifier,

      file_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      context_dependencies: Default::default(),
      diagnostics: Default::default(),
    };

    match self.module_factory.create(&mut create_data).await {
      Ok(result) => {
        if let Some(current_profile) = &factorize_task_result.current_profile {
          current_profile.mark_factory_end();
        }
        let diagnostics = create_data.diagnostics.drain(..).collect();
        Ok(TaskResult::Factorize(Box::new(
          factorize_task_result
            .with_factory_result(Some(result))
            .with_diagnostics(diagnostics)
            .with_file_dependencies(create_data.file_dependencies.drain())
            .with_missing_dependencies(create_data.missing_dependencies.drain())
            .with_context_dependencies(create_data.context_dependencies.drain()),
        )))
      }
      Err(mut e) => {
        if let Some(current_profile) = &factorize_task_result.current_profile {
          current_profile.mark_factory_end();
        }
        // Wrap source code if available
        if let Some(s) = self.original_module_source {
          e = e.with_source_code(s.source().to_string());
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if self.options.bail {
          return Err(e);
        }
        let mut diagnostics = Vec::with_capacity(create_data.diagnostics.len() + 1);
        diagnostics.push(e.into());
        diagnostics.append(&mut create_data.diagnostics);
        // Continue bundling if `options.bail` set to `false`.
        Ok(TaskResult::Factorize(Box::new(
          factorize_task_result
            .with_diagnostics(diagnostics)
            .with_file_dependencies(create_data.file_dependencies.drain())
            .with_missing_dependencies(create_data.missing_dependencies.drain())
            .with_context_dependencies(create_data.context_dependencies.drain()),
        )))
      }
    }
  }
}

pub type FactorizeQueue = WorkerQueue<FactorizeTask, DependencyId>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: Box<dyn Module>,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<DependencyId>,
  pub is_entry: bool,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub connect_origin: bool,
  #[derivative(Debug = "ignore")]
  pub callback: Option<ModuleCreationCallback>,
}

#[derive(Debug)]
pub enum AddTaskResult {
  ModuleReused {
    module: Box<dyn Module>,
  },
  ModuleAdded {
    module: Box<dyn Module>,
    current_profile: Option<Box<ModuleProfile>>,
  },
}

impl AddTask {
  pub fn run(self, compilation: &mut Compilation) -> Result<TaskResult> {
    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_integration_start();
    }

    if self.module.as_self_module().is_some() && self.connect_origin {
      let issuer = self
        .module_graph_module
        .get_issuer()
        .identifier()
        .expect("self module should have issuer");

      set_resolved_module(
        &mut compilation.get_module_graph_mut(),
        self.original_module_identifier,
        self.dependencies,
        *issuer,
      )?;

      return Ok(TaskResult::Add(Box::new(AddTaskResult::ModuleReused {
        module: self.module,
      })));
    }

    let module_identifier = self.module.identifier();

    if self.connect_origin
      && compilation
        .get_module_graph()
        .module_graph_module_by_identifier(&module_identifier)
        .is_some()
    {
      set_resolved_module(
        &mut compilation.get_module_graph_mut(),
        self.original_module_identifier,
        self.dependencies,
        module_identifier,
      )?;

      if let Some(callback) = self.callback {
        callback(&self.module);
      }

      return Ok(TaskResult::Add(Box::new(AddTaskResult::ModuleReused {
        module: self.module,
      })));
    }

    compilation
      .get_module_graph_mut()
      .add_module_graph_module(*self.module_graph_module);

    if self.connect_origin {
      set_resolved_module(
        &mut compilation.get_module_graph_mut(),
        self.original_module_identifier,
        self.dependencies,
        module_identifier,
      )?;
    }

    if self.is_entry {
      compilation
        .entry_module_identifiers
        .insert(module_identifier);
    }

    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_integration_end();
    }

    if let Some(callback) = self.callback {
      callback(&self.module);
    }

    Ok(TaskResult::Add(Box::new(AddTaskResult::ModuleAdded {
      module: self.module,
      current_profile: self.current_profile,
    })))
  }
}

fn set_resolved_module(
  module_graph: &mut ModuleGraph,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<DependencyId>,
  module_identifier: ModuleIdentifier,
) -> Result<()> {
  for dependency in dependencies {
    module_graph.set_resolved_module(original_module_identifier, dependency, module_identifier)?;
  }
  Ok(())
}

pub type AddQueue = WorkerQueue<AddTask, ModuleIdentifier>;

#[derive(Debug)]
pub struct BuildTask {
  pub module: Box<dyn Module>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub factorize_queue: Option<FactorizeQueueHandler>,
  pub add_queue: Option<AddQueueHandler>,
  pub build_queue: Option<BuildQueueHandler>,
  pub process_dependencies_queue: Option<ProcessDependenciesQueueHandler>,
  pub build_time_execution_queue: Option<BuildTimeExecutionQueueHandler>,
}

#[derive(Debug)]
pub struct BuildTaskResult {
  pub module: Box<dyn Module>,
  pub build_result: Box<BuildResult>,
  pub diagnostics: Vec<Diagnostic>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub from_cache: bool,
}

#[async_trait::async_trait]
impl WorkerTask for BuildTask {
  async fn run(self) -> Result<TaskResult> {
    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_building_start();
    }

    let mut module = self.module;
    let compiler_options = self.compiler_options;
    let resolver_factory = self.resolver_factory;
    let cache = self.cache;
    let plugin_driver = self.plugin_driver;

    let (build_result, is_cache_valid) = cache
      .build_module_occasion
      .use_cache(&mut module, |module| async {
        plugin_driver
          .compilation_hooks
          .build_module
          .call(module)
          .await?;

        let result = module
          .build(
            BuildContext {
              compiler_context: CompilerContext {
                options: compiler_options.clone(),
                resolver_factory: resolver_factory.clone(),
                module: module.identifier(),
                module_context: module.as_normal_module().and_then(|m| m.get_context()),
                module_source_map_kind: module.get_source_map_kind().clone(),
                factorize_queue: self.factorize_queue.clone(),
                add_queue: self.add_queue.clone(),
                build_queue: self.build_queue.clone(),
                process_dependencies_queue: self.process_dependencies_queue.clone(),
                build_time_execution_queue: self.build_time_execution_queue.clone(),
                plugin_driver: plugin_driver.clone(),
                cache: cache.clone(),
              },
              plugin_driver: plugin_driver.clone(),
              compiler_options: &compiler_options,
            },
            None,
          )
          .await;

        plugin_driver
          .compilation_hooks
          .succeed_module
          .call(module)
          .await?;

        result.map(|t| {
          let diagnostics = module
            .clone_diagnostics()
            .into_iter()
            .map(|d| d.with_module_identifier(Some(module.identifier())))
            .collect();
          (t.with_diagnostic(diagnostics), module)
        })
      })
      .await?;

    if is_cache_valid {
      plugin_driver
        .compilation_hooks
        .still_valid_module
        .call(&mut module)
        .await?;
    }

    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_building_end();
    }

    build_result.map(|build_result| {
      let (build_result, diagnostics) = build_result.split_into_parts();

      TaskResult::Build(Box::new(BuildTaskResult {
        module,
        build_result: Box::new(build_result),
        diagnostics,
        current_profile: self.current_profile,
        from_cache: is_cache_valid,
      }))
    })
  }
}

pub type BuildQueue = WorkerQueue<BuildTask, ModuleIdentifier>;

#[derive(Debug)]
pub struct ProcessDependenciesTask {
  pub original_module_identifier: ModuleIdentifier,
  pub dependencies: Vec<DependencyId>,
  pub resolve_options: Option<Box<Resolve>>,
}

#[derive(Debug)]
pub struct ProcessDependenciesResult {
  pub module_identifier: ModuleIdentifier,
}

pub type ProcessDependenciesQueue = WorkerQueue<ProcessDependenciesTask, ModuleIdentifier>;

#[derive(Clone, Debug)]
pub struct BuildTimeExecutionOption {
  pub public_path: Option<String>,
  pub base_uri: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildTimeExecutionTask {
  pub module: ModuleIdentifier,
  pub request: String,
  pub options: BuildTimeExecutionOption,
  pub sender: UnboundedSender<Result<ExecuteModuleResult>>,
}

pub type BuildTimeExecutionQueue = WorkerQueue<BuildTimeExecutionTask, ModuleIdentifier>;

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
    let module_graph = compilation.get_module_graph();
    let mgm = match module_graph.module_graph_module_by_identifier(&module_identifier) {
      Some(mgm) => mgm,
      None => {
        return CleanTaskResult::ModuleIsCleaned {
          module_identifier,
          dependent_module_identifiers: vec![],
        }
      }
    };

    if !mgm.incoming_connections().is_empty() {
      return CleanTaskResult::ModuleIsUsed { module_identifier };
    }

    let dependent_module_identifiers: Vec<ModuleIdentifier> = module_graph
      .get_module_all_depended_modules(&module_identifier)
      .expect("should have module")
      .into_iter()
      .copied()
      .collect();
    compilation
      .get_module_graph_mut()
      .revoke_module(&module_identifier);
    CleanTaskResult::ModuleIsCleaned {
      module_identifier,
      dependent_module_identifiers,
    }
  }
}
