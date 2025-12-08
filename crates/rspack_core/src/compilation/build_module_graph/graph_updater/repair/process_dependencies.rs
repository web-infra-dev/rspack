use std::borrow::Cow;

use rustc_hash::FxHashMap as HashMap;

use super::{TaskContext, factorize::FactorizeTask};
use crate::{
  ContextDependency, DependencyId, Module, ModuleIdentifier, ModuleProfile,
  utils::task_loop::{Task, TaskResult, TaskType},
};

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
    let mut sorted_dependencies = HashMap::default();
    let module_graph =
      &mut TaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);

    for dependency_id in dependencies {
      // Some dependencies here will not trigger the factorize task, so add all dependencies here.
      context
        .artifact
        .affected_dependencies
        .mark_as_add(&dependency_id);

      let dependency = module_graph
        .dependency_by_id(&dependency_id)
        .expect("should have dependency");
      // FIXME: now only module/context dependency can put into resolve queue.
      // FIXME: should align webpack
      let resource_identifier = if let Some(module_dependency) = dependency.as_module_dependency() {
        // TODO need implement more dependency `resource_identifier()`
        // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L1621
        let id = if let Some(resource_identifier) = module_dependency.resource_identifier() {
          Cow::Borrowed(resource_identifier)
        } else {
          Cow::Owned(format!(
            "{}|{}",
            module_dependency.dependency_type(),
            module_dependency.request()
          ))
        };
        Some(id)
      } else {
        dependency
          .as_context_dependency()
          .map(|d| Cow::Borrowed(ContextDependency::resource_identifier(d)))
      };

      if let Some(resource_identifier) = resource_identifier {
        sorted_dependencies
          .entry(resource_identifier)
          .or_insert(vec![])
          .push(dependency.clone());
      }
    }

    let module = module_graph
      .module_by_identifier(&original_module_identifier)
      .expect("Module expected");

    let mut res: Vec<Box<dyn Task<TaskContext>>> = vec![];
    for dependencies in sorted_dependencies.into_values() {
      let current_profile = context
        .compiler_options
        .profile
        .then(ModuleProfile::default);
      let original_module_source = module_graph
        .module_by_identifier(&original_module_identifier)
        .and_then(|m| m.as_normal_module())
        .and_then(|m| m.source().cloned());
      let dependency = &dependencies[0];
      let dependency_type = dependency.dependency_type();
      // TODO move module_factory calculate to dependency factories
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
        compiler_id: context.compiler_id,
        compilation_id: context.compilation_id,
        module_factory,
        original_module_identifier: Some(module.identifier()),
        original_module_context: module.get_context(),
        original_module_source,
        issuer: module
          .as_normal_module()
          .and_then(|module| module.name_for_condition()),
        issuer_layer: module.get_layer().cloned(),
        dependencies,
        resolve_options: module.get_resolve_options(),
        options: context.compiler_options.clone(),
        current_profile,
        resolver_factory: context.resolver_factory.clone(),
        from_unlazy,
      }));
    }
    Ok(res)
  }
}
