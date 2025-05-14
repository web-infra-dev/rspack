use std::collections::hash_map::Entry;

use super::{
  context::{ExecutorTaskContext, ImportModuleMeta},
  execute::{ExecuteResultSender, ExecuteTask},
  overwrite::overwrite_tasks,
};
use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Context, Dependency, LoaderImportDependency, ModuleProfile, PublicPath,
};

#[derive(Debug)]
pub struct EntryTask {
  pub meta: ImportModuleMeta,
  pub origin_module_context: Option<Context>,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub result_sender: ExecuteResultSender,
}
#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for EntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  async fn main_run(
    self: Box<Self>,
    context: &mut ExecutorTaskContext,
  ) -> TaskResult<ExecutorTaskContext> {
    let Self {
      meta,
      origin_module_context,
      public_path,
      base_uri,
      result_sender,
    } = *self;
    let ExecutorTaskContext {
      entries,
      origin_context,
      tracker,
      executed_entry_deps,
    } = context;

    let task = ExecuteTask {
      meta: meta.clone(),
      public_path,
      base_uri,
      result_sender,
    };
    let mut res = vec![];
    let dep_id = match entries.entry(meta.clone()) {
      Entry::Vacant(v) => {
        let dep = Box::new(LoaderImportDependency::new(
          meta.request,
          origin_module_context.unwrap_or(Context::from("")),
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
          issuer_layer: meta.layer.clone(),
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
        *v.insert(dep_id)
      }
      Entry::Occupied(v) => *v.get(),
    };

    executed_entry_deps.insert(dep_id);

    res.extend(
      tracker.on_entry(origin_context, dep_id, |error| match error {
        Some(error) => {
          task.finish_with_error(error);
          None
        }
        None => Some(Box::new(task)),
      }),
    );

    Ok(res)
  }
}
