use rustc_hash::FxHashMap as HashMap;

use super::{factorize::FactorizeTask, MakeTaskContext};
use crate::{
  utils::task_loop::{Task, TaskResult, TaskType},
  ContextDependency, DependencyId, DependencyType, ErrorSpan, Module, ModuleIdentifier,
  ModuleProfile, NormalModuleSource, Resolve,
};

#[derive(Debug)]
pub struct ProcessDependenciesTask {
  pub original_module_identifier: ModuleIdentifier,
  pub dependencies: Vec<DependencyId>,
  pub resolve_options: Option<Box<Resolve>>,
}

impl Task<MakeTaskContext> for ProcessDependenciesTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self {
      original_module_identifier,
      dependencies,
      resolve_options,
    } = *self;
    let mut sorted_dependencies = HashMap::default();
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut context.module_graph_partial);

    dependencies.into_iter().for_each(|dependency_id| {
      let dependency = module_graph
        .dependency_by_id(&dependency_id)
        .expect("should have dependency");
      // FIXME: now only module/context dependency can put into resolve queue.
      // FIXME: should align webpack
      let resource_identifier = if let Some(module_dependency) = dependency.as_module_dependency() {
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

    let module = module_graph
      .module_by_identifier(&original_module_identifier)
      .expect("Module expected");

    let mut res: Vec<Box<dyn Task<MakeTaskContext>>> = vec![];
    for dependencies in sorted_dependencies.into_values() {
      let current_profile = context
        .compiler_options
        .profile
        .then(Box::<ModuleProfile>::default);
      let dependency = module_graph
        .dependency_by_id(&dependencies[0])
        .expect("should have dependency")
        .clone();
      let original_module_source = module_graph
        .module_by_identifier(&original_module_identifier)
        .and_then(|m| m.as_normal_module())
        .and_then(|m| {
          if let NormalModuleSource::BuiltSucceed(s) = m.source() {
            Some(s.clone())
          } else {
            None
          }
        });
      let dependency_type = dependency.dependency_type();
      // TODO move module_factory calculate to dependency factories
      let module_factory = context
        .dependency_factories
        .get(&match dependency_type {
          DependencyType::EsmImport(_) => DependencyType::EsmImport(ErrorSpan::default()),
          DependencyType::EsmExport(_) => DependencyType::EsmExport(ErrorSpan::default()),
          _ => dependency_type.clone(),
        })
        .unwrap_or_else(|| {
          panic!(
            "No module factory available for dependency type: {}, resourceIdentifier: {:?}",
            dependency_type,
            dependency.resource_identifier()
          )
        })
        .clone();
      res.push(Box::new(FactorizeTask {
        module_factory,
        original_module_identifier: Some(module.identifier()),
        original_module_context: module.get_context(),
        original_module_source,
        issuer: module
          .as_normal_module()
          .and_then(|module| module.name_for_condition()),
        dependency,
        dependencies,
        is_entry: false,
        resolve_options: resolve_options.clone(),
        resolver_factory: context.resolver_factory.clone(),
        loader_resolver_factory: context.loader_resolver_factory.clone(),
        options: context.compiler_options.clone(),
        plugin_driver: context.plugin_driver.clone(),
        cache: context.cache.clone(),
        current_profile,
      }));
    }
    Ok(res)
  }
}
