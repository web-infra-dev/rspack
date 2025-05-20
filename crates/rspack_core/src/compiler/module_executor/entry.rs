use std::collections::hash_map::Entry;

use rspack_error::miette::diagnostic;

use super::{
  context::{ExecutorTaskContext, ImportModuleEntry, ImportModuleMeta},
  execute::{ExecuteResultSender, ExecuteTask},
  overwrite::overwrite_tasks,
};
use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Dependency, LoaderImportDependency, ModuleIdentifier, ModuleProfile, PublicPath,
};

/// A task for generate import module entry.
#[derive(Debug)]
pub struct EntryTask {
  pub meta: ImportModuleMeta,
  pub origin_module_identifier: ModuleIdentifier,
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
      origin_module_identifier,
      public_path,
      base_uri,
      result_sender,
    } = *self;
    let ExecutorTaskContext {
      entries,
      origin_context,
      tracker,
      used_entry,
    } = context;

    let task = ExecuteTask {
      meta: meta.clone(),
      public_path,
      base_uri,
      result_sender,
    };
    let mut res = vec![];
    let (dep_id, is_new) = match entries.entry(meta.clone()) {
      Entry::Vacant(v) => {
        // not exist, generate a new dependency
        let dep = Box::new(LoaderImportDependency::new(
          meta.request,
          meta.origin_module_context,
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
        let value = v.insert(ImportModuleEntry {
          dep_id,
          origin_module_identifiers: Default::default(),
        });
        value
          .origin_module_identifiers
          .insert(origin_module_identifier);
        (value.dep_id, true)
      }
      Entry::Occupied(mut v) => {
        let value = v.get_mut();
        value
          .origin_module_identifiers
          .insert(origin_module_identifier);
        (value.dep_id, false)
      }
    };

    // mark as used
    let used_origin_mids = used_entry.entry(dep_id).or_default();
    if used_origin_mids.contains(&origin_module_identifier) {
      task.finish_with_error(diagnostic!("module {} use importModule with same path multi times, maybe have a circular build dependency", origin_module_identifier).into());
      return Ok(vec![]);
    } else {
      used_origin_mids.insert(origin_module_identifier);
    }

    res.extend(tracker.on_entry(dep_id, Box::new(task), is_new));

    Ok(res)
  }
}
