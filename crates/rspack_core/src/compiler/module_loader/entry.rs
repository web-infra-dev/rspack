use std::collections::hash_map::Entry;

use rspack_error::miette::diagnostic;

use super::{
  context::{LoadModuleMeta, LoadTaskContext},
  execute::ExecuteTask,
  overwrite::overwrite_tasks,
};
use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Context, Dependency, LoaderLoadDependency, ModuleProfile,
};

/// A task for generate import module entry.
#[derive(Debug)]
pub struct EntryTask {
  pub meta: LoadModuleMeta,
  pub origin_module_context: Context,
  pub execute_task: ExecuteTask,
}
#[async_trait::async_trait]
impl Task<LoadTaskContext> for EntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut LoadTaskContext) -> TaskResult<LoadTaskContext> {
    let Self {
      meta,
      origin_module_context,
      execute_task,
    } = *self;
    let LoadTaskContext {
      entries,
      origin_context,
      tracker,
      used_entry,
    } = context;

    let mut res = vec![];
    let (dep_id, is_new) = match entries.entry(meta.clone()) {
      Entry::Vacant(v) => {
        // not exist, generate a new dependency
        let dep = Box::new(LoaderLoadDependency::new(
          meta.request.clone(),
          origin_module_context,
        ));
        let dep_id = *dep.id();

        let mut mg =
          MakeTaskContext::get_module_graph_mut(&mut origin_context.artifact.module_graph_partial);
        mg.add_dependency(dep.clone());

        res.extend(overwrite_tasks(vec![Box::new(FactorizeTask {
          compiler_id: origin_context.compiler_id,
          compilation_id: origin_context.compilation_id,
          module_factory: origin_context
            .dependency_factories
            .get(dep.dependency_type())
            .unwrap_or_else(|| {
              panic!(
                "should have dependency_factories for dependency_type: {}",
                dep.dependency_type()
              )
            })
            .clone(),
          original_module_identifier: None,
          original_module_source: None,
          issuer: None,
          issuer_layer: None,
          original_module_context: None,
          dependencies: vec![dep],
          resolve_options: None,
          options: origin_context.compiler_options.clone(),
          current_profile: origin_context
            .compiler_options
            .profile
            .then(Box::<ModuleProfile>::default),
          resolver_factory: origin_context.resolver_factory.clone(),
        })]));
        (*v.insert(dep_id), true)
      }
      Entry::Occupied(v) => (*v.get(), false),
    };

    // mark as used
    used_entry.insert(dep_id);

    if tracker.is_running(&dep_id) {
      let mg = origin_context.artifact.get_module_graph();
      // the module in module loader need to check.
      if mg
        .module_graph_module_by_identifier(&meta.origin_module_identifier)
        .is_some()
      {
        execute_task.finish_with_error(
          diagnostic!(
            "The added task is running, maybe have a circular build dependency. MetaInfo: {:?}",
            meta
          )
          .into(),
        );
        return Ok(vec![]);
      }
    }

    res.extend(tracker.on_entry(is_new, dep_id, Box::new(execute_task)));

    Ok(res)
  }
}
