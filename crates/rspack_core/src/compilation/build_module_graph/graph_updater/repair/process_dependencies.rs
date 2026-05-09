use std::sync::Arc;

use rspack_sources::BoxSource;
use rustc_hash::FxHashMap as HashMap;

use super::{TaskContext, factorize::FactorizeTask};
use crate::{
  BoxDependency, CompilerOptions, Context, ContextDependency, DependencyId, Module,
  ModuleIdentifier, ModuleLayer, Resolve, ResolverFactory,
  dependency::DependencyType,
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DependencyResourceKey<'a> {
  Resource(&'a str),
  TypedRequest(DependencyType, &'a str),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum OwnedDependencyResourceKey {
  Resource(Box<str>),
  TypedRequest(DependencyType, Box<str>),
}

impl From<DependencyResourceKey<'_>> for OwnedDependencyResourceKey {
  fn from(value: DependencyResourceKey<'_>) -> Self {
    match value {
      DependencyResourceKey::Resource(resource_identifier) => {
        Self::Resource(resource_identifier.into())
      }
      DependencyResourceKey::TypedRequest(dependency_type, request) => {
        Self::TypedRequest(dependency_type, request.into())
      }
    }
  }
}

fn dependency_resource_key(dependency: &BoxDependency) -> Option<DependencyResourceKey<'_>> {
  if let Some(module_dependency) = dependency.as_module_dependency() {
    Some(
      module_dependency
        .resource_identifier()
        .map(DependencyResourceKey::Resource)
        .unwrap_or_else(|| {
          DependencyResourceKey::TypedRequest(
            *module_dependency.dependency_type(),
            module_dependency.request(),
          )
        }),
    )
  } else {
    dependency
      .as_context_dependency()
      .map(|d| DependencyResourceKey::Resource(ContextDependency::resource_identifier(d)))
  }
}

pub type FactorizeDependencyGroups = Vec<Vec<BoxDependency>>;

#[derive(Debug)]
pub struct FactorizeTaskModuleContext {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub original_module_source: Option<BoxSource>,
  pub original_module_context: Option<Box<Context>>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<ModuleLayer>,
  pub resolve_options: Option<Arc<Resolve>>,
}

#[derive(Debug)]
pub struct FactorizeTaskSharedContext {
  pub compiler_id: crate::CompilerId,
  pub compilation_id: crate::CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
}

pub fn push_factorize_dependency(
  grouped_dependencies: &mut HashMap<OwnedDependencyResourceKey, Vec<BoxDependency>>,
  dependency: BoxDependency,
) {
  if let Some(resource_key) = dependency_resource_key(&dependency) {
    grouped_dependencies
      .entry(resource_key.into())
      .or_default()
      .push(dependency);
  }
}

pub fn create_factorize_tasks(
  context: &TaskContext,
  shared_context: FactorizeTaskSharedContext,
  module_context: &FactorizeTaskModuleContext,
  dependency_groups: FactorizeDependencyGroups,
  from_unlazy: bool,
) -> Vec<Box<dyn Task<TaskContext>>> {
  let mut res: Vec<Box<dyn Task<TaskContext>>> = Vec::with_capacity(dependency_groups.len());
  for dependencies in dependency_groups {
    let dependency = &dependencies[0];
    let dependency_type = dependency.dependency_type();
    let module_factory = context
      .dependency_factories
      .get(dependency_type)
      .unwrap_or_else(|| {
        panic!(
          "No module factory available for dependency type: {}, resourceIdentifier: {:?}",
          dependency_type,
          dependency.resource_identifier()
        )
      })
      .clone();
    res.push(Box::new(FactorizeTask {
      compiler_id: shared_context.compiler_id,
      compilation_id: shared_context.compilation_id,
      module_factory,
      original_module_identifier: module_context.original_module_identifier,
      original_module_context: module_context.original_module_context.clone(),
      original_module_source: module_context.original_module_source.clone(),
      issuer: module_context.issuer.clone(),
      issuer_layer: module_context.issuer_layer.clone(),
      dependencies,
      resolve_options: module_context.resolve_options.clone(),
      options: shared_context.options.clone(),
      resolver_factory: shared_context.resolver_factory.clone(),
      from_unlazy,
    }));
  }
  res
}

#[derive(Debug)]
pub struct ProcessDependenciesTask {
  pub original_module_identifier: ModuleIdentifier,
  pub dependencies: Vec<DependencyId>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for ProcessDependenciesTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let Self {
      original_module_identifier,
      dependencies,
      from_unlazy,
    } = *self;
    let mut sorted_dependencies: HashMap<DependencyResourceKey<'_>, Vec<BoxDependency>> =
      HashMap::default();

    // First mark all dependencies as added
    for dependency_id in &dependencies {
      context
        .artifact
        .affected_dependencies
        .mark_as_add(dependency_id);
    }

    let module_graph = &mut context.artifact.module_graph;

    for dependency_id in dependencies {
      let dependency = module_graph.dependency_by_id(&dependency_id);
      if let Some(resource_identifier) = dependency_resource_key(dependency) {
        sorted_dependencies
          .entry(resource_identifier)
          .or_default()
          .push(dependency.clone());
      }
    }

    let module = module_graph
      .module_by_identifier(&original_module_identifier)
      .expect("Module expected");
    let module_context = FactorizeTaskModuleContext {
      original_module_identifier: Some(module.identifier()),
      original_module_source: module.as_normal_module().and_then(|m| m.source().cloned()),
      original_module_context: module.get_context(),
      issuer: module
        .as_normal_module()
        .and_then(|module| module.name_for_condition()),
      issuer_layer: module.get_layer().cloned(),
      resolve_options: module.get_resolve_options(),
    };

    let dependency_groups = sorted_dependencies.into_values().collect();

    Ok(create_factorize_tasks(
      context,
      FactorizeTaskSharedContext {
        compiler_id: context.compiler_id,
        compilation_id: context.compilation_id,
        options: context.compiler_options.clone(),
        resolver_factory: context.resolver_factory.clone(),
      },
      &module_context,
      dependency_groups,
      from_unlazy,
    ))
  }
}
