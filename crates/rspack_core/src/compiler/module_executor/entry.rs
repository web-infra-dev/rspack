use tokio::sync::mpsc::UnboundedSender;

use super::{ctrl::Event, overwrite::OverwriteTask};
use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Dependency, DependencyId, LoaderImportDependency, ModuleProfile,
};

#[derive(Debug)]
pub enum EntryParam {
  DependencyId(DependencyId),
  Entry(Box<LoaderImportDependency>),
}

#[derive(Debug)]
pub struct EntryTask {
  pub param: EntryParam,
  pub event_sender: UnboundedSender<Event>,
}

impl Task<MakeTaskContext> for EntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self {
      param,
      event_sender,
    } = *self;
    let mut module_graph =
      MakeTaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);

    match param {
      EntryParam::DependencyId(dep_id) => {
        if let Some(module_id) = module_graph.module_identifier_by_dependency_id(&dep_id) {
          event_sender
            .send(Event::FinishDeps(Some(*module_id)))
            .expect("should success");
        }
        Ok(vec![])
      }
      EntryParam::Entry(dep) => {
        module_graph.add_dependency(dep.clone());
        let task = Box::new(FactorizeTask {
          module_factory: context
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
          options: context.compiler_options.clone(),
          current_profile: context
            .compiler_options
            .profile
            .then(Box::<ModuleProfile>::default),
        });
        Ok(vec![Box::new(OverwriteTask {
          origin_task: task,
          event_sender,
        })])
      }
    }
  }
}
