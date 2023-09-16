use std::sync::Arc;

use rspack_error::{Diagnostic, Result};

use crate::{
  cache::Cache, BoxDependency, BuildContext, BuildResult, Compilation, CompilerContext,
  CompilerOptions, Context, ContextModuleFactory, DependencyType, Module, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ModuleGraph, ModuleGraphModule, ModuleIdentifier,
  ModuleProfile, ModuleType, NormalModuleFactory, NormalModuleFactoryContext, Resolve,
  ResolverFactory, SharedPluginDriver, WorkerQueue,
};
use crate::{ExportInfo, ExportsInfo, UsageState};

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

pub struct FactorizeTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub dependencies: Vec<BoxDependency>,
  pub is_entry: bool,
  pub module_type: Option<ModuleType>,
  pub side_effects: Option<bool>,
  pub resolve_options: Option<Box<Resolve>>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub loader_resolver_factory: Arc<ResolverFactory>,
  pub options: Arc<CompilerOptions>,
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
  pub current_profile: Option<Box<ModuleProfile>>,
}

/// a struct temporarily used creating ExportsInfo
#[derive(Debug)]
pub struct ExportsInfoRelated {
  pub exports_info: ExportsInfo,
  pub other_exports_info: ExportInfo,
  pub side_effects_info: ExportInfo,
}
#[derive(Debug)]
pub struct FactorizeTaskResult {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub factory_result: ModuleFactoryResult,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxDependency>,
  pub diagnostics: Vec<Diagnostic>,
  pub is_entry: bool,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub exports_info_related: ExportsInfoRelated,
  pub from_cache: bool,
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeTask {
  async fn run(self) -> Result<TaskResult> {
    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_factory_start();
    }
    let dependency = &self.dependencies[0];

    let context = if let Some(context) = dependency.get_context() {
      context
    } else if let Some(context) = &self.original_module_context {
      context
    } else {
      &self.options.context
    }
    .clone();

    let (result, diagnostics) = match *dependency.dependency_type() {
      DependencyType::ImportContext
      | DependencyType::CommonJSRequireContext
      | DependencyType::RequireContext => {
        let factory = ContextModuleFactory::new(self.plugin_driver, self.cache);
        factory
          .create(ModuleFactoryCreateData {
            resolve_options: self.resolve_options,
            context,
            dependency: dependency.clone(),
          })
          .await?
          .split_into_parts()
      }
      _ => {
        let factory = NormalModuleFactory::new(
          NormalModuleFactoryContext {
            original_module_identifier: self.original_module_identifier,
            module_type: self.module_type,
            side_effects: self.side_effects,
            options: self.options.clone(),
            lazy_visit_modules: self.lazy_visit_modules,
            issuer: self.issuer,
          },
          self.loader_resolver_factory,
          self.plugin_driver,
          self.cache,
        );
        factory
          .create(ModuleFactoryCreateData {
            resolve_options: self.resolve_options,
            context,
            dependency: dependency.clone(),
          })
          .await?
          .split_into_parts()
      }
    };

    let other_exports_info = ExportInfo::new(None, UsageState::Unknown, None);
    let side_effects_only_info = ExportInfo::new(
      Some("*side effects only*".into()),
      UsageState::Unknown,
      None,
    );
    let exports_info = ExportsInfo::new(other_exports_info.id, side_effects_only_info.id);
    let mgm = ModuleGraphModule::new(
      result.module.identifier(),
      *result.module.module_type(),
      exports_info.id,
    );

    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_factory_end();
    }

    Ok(TaskResult::Factorize(Box::new(FactorizeTaskResult {
      is_entry: self.is_entry,
      original_module_identifier: self.original_module_identifier,
      from_cache: result.from_cache,
      factory_result: result,
      module_graph_module: Box::new(mgm),
      dependencies: self.dependencies,
      diagnostics,
      current_profile: self.current_profile,
      exports_info_related: ExportsInfoRelated {
        exports_info,
        other_exports_info,
        side_effects_info: side_effects_only_info,
      },
    })))
  }
}

pub type FactorizeQueue = WorkerQueue<FactorizeTask>;

pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: Box<dyn Module>,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxDependency>,
  pub is_entry: bool,
  pub current_profile: Option<Box<ModuleProfile>>,
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
    let module_identifier = self.module.identifier();

    if compilation
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .is_some()
    {
      Self::set_resolved_module(
        &mut compilation.module_graph,
        self.original_module_identifier,
        self.dependencies,
        module_identifier,
      )?;

      return Ok(TaskResult::Add(Box::new(AddTaskResult::ModuleReused {
        module: self.module,
      })));
    }

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

    if let Some(current_profile) = &self.current_profile {
      current_profile.mark_integration_end();
    }

    Ok(TaskResult::Add(Box::new(AddTaskResult::ModuleAdded {
      module: self.module,
      current_profile: self.current_profile,
    })))
  }
}

impl AddTask {
  fn set_resolved_module(
    module_graph: &mut ModuleGraph,
    original_module_identifier: Option<ModuleIdentifier>,
    dependencies: Vec<BoxDependency>,
    module_identifier: ModuleIdentifier,
  ) -> Result<()> {
    for dependency in dependencies {
      module_graph.set_resolved_module(
        original_module_identifier,
        dependency,
        module_identifier,
      )?;
    }
    Ok(())
  }
}

pub type AddQueue = WorkerQueue<AddTask>;

pub struct BuildTask {
  pub module: Box<dyn Module>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
  pub current_profile: Option<Box<ModuleProfile>>,
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

    let (build_result, is_cache_valid) = match cache
      .build_module_occasion
      .use_cache(&mut module, |module| async {
        plugin_driver
          .build_module(module.as_mut())
          .await
          .unwrap_or_else(|e| panic!("Run build_module hook failed: {}", e));

        let result = module
          .build(BuildContext {
            compiler_context: CompilerContext {
              options: compiler_options.clone(),
              resolver_factory: resolver_factory.clone(),
            },
            plugin_driver: plugin_driver.clone(),
            compiler_options: &compiler_options,
          })
          .await;

        plugin_driver
          .succeed_module(&**module)
          .await
          .unwrap_or_else(|e| panic!("Run succeed_module hook failed: {}", e));

        result.map(|t| (t, module))
      })
      .await
    {
      Ok(result) => result,
      Err(err) => panic!("build module get error: {}", err),
    };

    if is_cache_valid {
      plugin_driver.still_valid_module(module.as_ref()).await?;
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

pub type BuildQueue = WorkerQueue<BuildTask>;

pub struct ProcessDependenciesTask {
  pub original_module_identifier: ModuleIdentifier,
  pub dependencies: Vec<BoxDependency>,
  pub resolve_options: Option<Box<Resolve>>,
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
      .into_iter()
      .copied()
      .collect();
    compilation.module_graph.revoke_module(&module_identifier);
    CleanTaskResult::ModuleIsCleaned {
      module_identifier,
      dependent_module_identifiers,
    }
  }
}

pub type CleanQueue = WorkerQueue<CleanTask>;
