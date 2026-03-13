use rspack_error::Result;

use super::{TaskContext, build::BuildTask, lazy::process_unlazy_dependencies};
use crate::{
  BoxDependency, BoxModule, BuildModuleGraphArtifact, FactorizeInfo, ModuleIdentifier,
  NormalModuleDedupeWaiter,
  compilation::build_module_graph::ForwardedIdSet,
  module_graph::{ModuleGraph, ModuleGraphModule},
  utils::{
    ResourceId,
    task_loop::{Task, TaskResult, TaskType},
  },
};

#[derive(Debug)]
pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: BoxModule,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxDependency>,
  pub from_unlazy: bool,
}

#[derive(Debug)]
pub struct ReuseNormalModuleTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module_identifier: ModuleIdentifier,
  pub dependencies: Vec<BoxDependency>,
  pub factorize_info: FactorizeInfo,
  pub from_unlazy: bool,
}

impl ReuseNormalModuleTask {
  pub fn from_waiter(
    module_identifier: ModuleIdentifier,
    waiter: NormalModuleDedupeWaiter,
  ) -> Self {
    Self {
      original_module_identifier: waiter.original_module_identifier,
      module_identifier,
      dependencies: waiter.dependencies,
      factorize_info: waiter.factorize_info,
      from_unlazy: waiter.from_unlazy,
    }
  }
}

#[async_trait::async_trait]
impl Task<TaskContext> for AddTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let module_identifier = self.module.identifier();

    // reuse module for self referenced module
    if self.module.as_self_module().is_some() {
      let issuer = self
        .module_graph_module
        .issuer()
        .identifier()
        .expect("self module should have issuer");

      set_resolved_module(
        &mut context.artifact.module_graph,
        self.original_module_identifier,
        self.dependencies,
        *issuer,
      )?;

      return Ok(vec![]);
    }

    let forwarded_ids = ForwardedIdSet::from_dependencies(&self.dependencies);
    let is_normal_module = self.module.as_normal_module().is_some();

    // reuse module if module is already added by other dependency
    if context
      .artifact
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .is_some()
    {
      let mut tasks = resolve_to_existing_module(
        context,
        self.original_module_identifier,
        self.dependencies,
        module_identifier,
        self.from_unlazy,
      )?;
      if is_normal_module {
        tasks.extend(take_normal_module_dedupe_tasks(context, module_identifier));
      }
      return Ok(tasks);
    }

    context
      .artifact
      .module_graph
      .add_module_graph_module(*self.module_graph_module);

    context
      .exports_info_artifact
      .new_exports_info(module_identifier);

    set_resolved_module(
      &mut context.artifact.module_graph,
      self.original_module_identifier,
      self.dependencies,
      module_identifier,
    )?;

    tracing::trace!("Module added: {}", self.module.identifier());
    context
      .artifact
      .affected_modules
      .mark_as_add(&module_identifier);
    let mut tasks: Vec<Box<dyn Task<TaskContext>>> = vec![Box::new(BuildTask {
      compiler_id: context.compiler_id,
      compilation_id: context.compilation_id,
      module: self.module,
      resolver_factory: context.resolver_factory.clone(),
      compiler_options: context.compiler_options.clone(),
      plugin_driver: context.plugin_driver.clone(),
      runtime_template: context.runtime_template.create_module_code_template(),
      fs: context.fs.clone(),
      forwarded_ids,
    })];
    if is_normal_module {
      tasks.extend(take_normal_module_dedupe_tasks(context, module_identifier));
    }
    Ok(tasks)
  }
}

#[async_trait::async_trait]
impl Task<TaskContext> for ReuseNormalModuleTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let ReuseNormalModuleTask {
      original_module_identifier,
      module_identifier,
      mut dependencies,
      mut factorize_info,
      from_unlazy,
    } = *self;

    apply_factorize_info(
      &mut context.artifact,
      &mut dependencies,
      &mut factorize_info,
    );
    let tasks = resolve_to_existing_module(
      context,
      original_module_identifier,
      dependencies,
      module_identifier,
      from_unlazy,
    )?;
    Ok(tasks)
  }
}

pub(super) fn apply_factorize_info(
  artifact: &mut BuildModuleGraphArtifact,
  dependencies: &mut [BoxDependency],
  factorize_info: &mut FactorizeInfo,
) {
  if !factorize_info.is_success() {
    artifact
      .make_failed_dependencies
      .insert(*dependencies[0].id());
  }
  let resource_id = ResourceId::from(*dependencies[0].id());
  artifact
    .file_dependencies
    .add_files(&resource_id, factorize_info.file_dependencies());
  artifact
    .context_dependencies
    .add_files(&resource_id, factorize_info.context_dependencies());
  artifact
    .missing_dependencies
    .add_files(&resource_id, factorize_info.missing_dependencies());

  for dep in dependencies {
    artifact.affected_dependencies.mark_as_add(dep.id());

    let dep_factorize_info = if let Some(d) = dep.as_context_dependency_mut() {
      d.factorize_info_mut()
    } else if let Some(d) = dep.as_module_dependency_mut() {
      d.factorize_info_mut()
    } else {
      unreachable!("only module dependency and context dependency can factorize")
    };
    *dep_factorize_info = std::mem::take(factorize_info);
  }
}

pub(super) fn resolve_to_existing_module(
  context: &mut TaskContext,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<BoxDependency>,
  module_identifier: ModuleIdentifier,
  from_unlazy: bool,
) -> Result<Vec<Box<dyn Task<TaskContext>>>> {
  let module_graph = &mut context.artifact.module_graph;
  let forwarded_ids = ForwardedIdSet::from_dependencies(&dependencies);
  set_resolved_module(
    module_graph,
    original_module_identifier,
    dependencies,
    module_identifier,
  )?;

  if from_unlazy {
    context
      .artifact
      .affected_modules
      .mark_as_add(&module_identifier);
  }

  if module_graph
    .module_by_identifier(&module_identifier)
    .is_some()
  {
    if context
      .artifact
      .module_to_lazy_make
      .has_lazy_dependencies(&module_identifier)
      && !forwarded_ids.is_empty()
    {
      if let Some(task) = process_unlazy_dependencies(
        &context.artifact.module_to_lazy_make,
        module_graph,
        forwarded_ids,
        module_identifier,
      ) {
        return Ok(vec![Box::new(task)]);
      }
    }
  } else {
    let pending_forwarded_ids = context
      .artifact
      .module_to_lazy_make
      .pending_forwarded_ids(module_identifier);
    pending_forwarded_ids.append(forwarded_ids);
  }

  Ok(vec![])
}

fn take_normal_module_dedupe_tasks(
  context: &TaskContext,
  module_identifier: ModuleIdentifier,
) -> Vec<Box<dyn Task<TaskContext>>> {
  context
    .normal_module_dedupe_tracker
    .mark_ready(module_identifier)
    .into_iter()
    .map(|waiter| {
      Box::new(ReuseNormalModuleTask::from_waiter(
        module_identifier,
        waiter,
      )) as Box<dyn Task<TaskContext>>
    })
    .collect()
}

fn set_resolved_module(
  module_graph: &mut ModuleGraph,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<BoxDependency>,
  module_identifier: ModuleIdentifier,
) -> Result<()> {
  for dependency in dependencies {
    module_graph.set_resolved_module(
      original_module_identifier,
      *dependency.id(),
      module_identifier,
    )?;
    module_graph.add_dependency(dependency);
  }
  Ok(())
}
