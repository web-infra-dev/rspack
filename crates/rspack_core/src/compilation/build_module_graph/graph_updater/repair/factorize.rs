use std::sync::Arc;

use rspack_error::Diagnostic;
use rspack_sources::BoxSource;

use super::{
  TaskContext,
  add::{AddTask, ReuseNormalModuleTask, apply_factorize_info},
};
use crate::{
  BoxDependency, CompilationId, CompilerId, CompilerOptions, Context, FactorizeInfo, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, ModuleLayer,
  NormalModuleDedupeTracker, NormalModuleDedupeWaiter, Resolve, ResolverFactory,
  module_graph::ModuleGraphModule,
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct FactorizeTask {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub module_factory: Arc<dyn ModuleFactory>,
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_source: Option<BoxSource>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<ModuleLayer>,
  pub dependencies: Vec<BoxDependency>,
  pub resolve_options: Option<Arc<Resolve>>,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub normal_module_dedupe_tracker: Arc<NormalModuleDedupeTracker>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }
  async fn background_run(mut self: Box<Self>) -> TaskResult<TaskContext> {
    let dependency = &self.dependencies[0];

    let context = if let Some(context) = dependency.get_context()
      && !context.is_empty()
    {
      context
    } else if let Some(context) = &self.original_module_context
      && !context.is_empty()
    {
      context
    } else {
      &self.options.context
    }
    .clone();

    let issuer_layer = dependency
      .get_layer()
      .or(self.issuer_layer.as_ref())
      .cloned();

    let request = self.dependencies[0]
      .as_module_dependency()
      .map(|d| d.request().to_string())
      .or_else(|| {
        self.dependencies[0]
          .as_context_dependency()
          .map(|d| d.request().to_string())
      })
      .unwrap_or_default();
    // Error and result are not mutually exclusive in webpack module factorization.
    // Rspack puts results that need to be shared in both error and ok in [ModuleFactoryCreateData].
    let mut create_data = ModuleFactoryCreateData {
      compiler_id: self.compiler_id,
      compilation_id: self.compilation_id,
      resolve_options: self.resolve_options,
      options: self.options.clone(),
      context,
      request,
      dependencies: self.dependencies,
      issuer: self.issuer,
      issuer_identifier: self.original_module_identifier,
      issuer_layer,
      claimed_normal_module_identifier: None,
      resolver_factory: self.resolver_factory,
      normal_module_dedupe_tracker: self.normal_module_dedupe_tracker,

      file_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      context_dependencies: Default::default(),
      diagnostics: Default::default(),
    };
    let factory_result = match self.module_factory.create(&mut create_data).await {
      Ok(result) => Some(result),
      Err(mut e) => {
        // Wrap source code if available
        if let Some(s) = self.original_module_source {
          let has_source_code = e.src.is_some();
          if !has_source_code {
            e.src = Some(s.source().into_string_lossy().into_owned());
          }
        }
        // Bail out if `options.bail` set to `true`,
        // which means 'Fail out on the first error instead of tolerating it.'
        if self.options.bail {
          return Err(e);
        }
        let mut diagnostic = Diagnostic::from(e);
        diagnostic.loc = create_data.dependencies[0].loc();
        create_data.diagnostics.insert(0, diagnostic);
        None
      }
    };

    let normal_module_dedup = factory_result
      .as_ref()
      .and_then(|result| result.normal_module_dedup);

    let factorize_info = FactorizeInfo::new(
      create_data.diagnostics,
      create_data
        .dependencies
        .iter()
        .map(|dep| *dep.id())
        .collect(),
      create_data.file_dependencies,
      create_data.context_dependencies,
      create_data.missing_dependencies,
    );

    let factorize_failed = factory_result.is_none();
    let waiter_failure_info = factorize_failed.then_some(factorize_info.clone());
    let claimed_normal_module_identifier = create_data.claimed_normal_module_identifier;

    let mut tasks: Vec<Box<dyn Task<TaskContext>>> = vec![Box::new(FactorizeResultTask {
      original_module_identifier: self.original_module_identifier,
      factory_result,
      normal_module_dedup,
      dependencies: create_data.dependencies,
      factorize_info,
      from_unlazy: self.from_unlazy,
    })];

    if factorize_failed
      && let Some(module_identifier) = claimed_normal_module_identifier
      && let Some(waiter_failure_info) = waiter_failure_info.as_ref()
    {
      for waiter in create_data
        .normal_module_dedupe_tracker
        .take_waiters_and_clear(module_identifier)
      {
        let factorize_info = clone_factorize_failure_info(waiter_failure_info, &waiter);
        tasks.push(Box::new(FactorizeResultTask {
          original_module_identifier: waiter.original_module_identifier,
          factory_result: None,
          normal_module_dedup: None,
          dependencies: waiter.dependencies,
          factorize_info,
          from_unlazy: waiter.from_unlazy,
        }));
      }
    }

    Ok(tasks)
  }
}

#[derive(Debug)]
pub struct FactorizeResultTask {
  //  pub dependency: DependencyId,
  pub original_module_identifier: Option<ModuleIdentifier>,
  /// Result will be available if [crate::ModuleFactory::create] returns `Ok`.
  pub factory_result: Option<ModuleFactoryResult>,
  pub normal_module_dedup: Option<ModuleIdentifier>,
  pub dependencies: Vec<BoxDependency>,
  pub factorize_info: FactorizeInfo,
  pub from_unlazy: bool,
}

#[derive(Debug)]
pub struct PendingNormalModuleDedupeTask;

#[async_trait::async_trait]
impl Task<TaskContext> for PendingNormalModuleDedupeTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, _context: &mut TaskContext) -> TaskResult<TaskContext> {
    Ok(vec![])
  }
}

fn clone_factorize_failure_info(
  factorize_info: &FactorizeInfo,
  waiter: &NormalModuleDedupeWaiter,
) -> FactorizeInfo {
  let mut diagnostics = factorize_info.diagnostics().to_vec();
  if let Some(diagnostic) = diagnostics.first_mut() {
    diagnostic.loc = waiter.dependencies[0].loc();
  }

  FactorizeInfo::new(
    diagnostics,
    waiter.dependencies.iter().map(|dep| *dep.id()).collect(),
    waiter.factorize_info.file_dependencies().clone(),
    waiter.factorize_info.context_dependencies().clone(),
    waiter.factorize_info.missing_dependencies().clone(),
  )
}

#[async_trait::async_trait]
impl Task<TaskContext> for FactorizeResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let FactorizeResultTask {
      original_module_identifier,
      factory_result,
      normal_module_dedup,
      mut dependencies,
      mut factorize_info,
      from_unlazy,
    } = *self;

    if let Some(module_identifier) = normal_module_dedup {
      let waiter = NormalModuleDedupeWaiter {
        original_module_identifier,
        dependencies,
        factorize_info,
        from_unlazy,
      };
      if let Some(waiter) = context
        .normal_module_dedupe_tracker
        .register_waiter_or_get_ready(module_identifier, waiter)
      {
        return Ok(vec![Box::new(ReuseNormalModuleTask::from_waiter(
          module_identifier,
          waiter,
        ))]);
      }
      return Ok(vec![Box::new(PendingNormalModuleDedupeTask)]);
    }

    apply_factorize_info(
      &mut context.artifact,
      &mut dependencies,
      &mut factorize_info,
    );

    let module_graph = context.artifact.get_module_graph_mut();
    let Some(factory_result) = factory_result else {
      let dep = &dependencies[0];
      tracing::trace!("Module created with failure, but without bailout: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency(dep)
      }
      return Ok(vec![]);
    };

    let Some(module) = factory_result.module else {
      let dep = &dependencies[0];
      tracing::trace!("Module ignored: {dep:?}");
      // sync dependencies to mg
      for dep in dependencies {
        module_graph.add_dependency(dep)
      }
      return Ok(vec![]);
    };
    let module_identifier = module.identifier();
    let mut mgm = ModuleGraphModule::new(module.identifier());
    mgm.set_issuer_if_unset(original_module_identifier);

    tracing::trace!("Module created: {}", &module_identifier);

    Ok(vec![Box::new(AddTask {
      original_module_identifier,
      module,
      module_graph_module: Box::new(mgm),
      dependencies,
      from_unlazy,
    })])
  }
}
