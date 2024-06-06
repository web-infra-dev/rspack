use tokio::sync::mpsc::UnboundedSender;

use super::ctrl::Event;
use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Dependency, DependencyId, EntryDependency, ModuleProfile,
};

#[derive(Debug)]
pub enum EntryParam {
  DependencyId(DependencyId, UnboundedSender<Event>),
  EntryDependency(Box<EntryDependency>),
}

#[derive(Debug)]
pub struct EntryTask {
  pub param: EntryParam,
}

impl Task<MakeTaskContext> for EntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self { param } = *self;
    let mut module_graph =
      MakeTaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);

    match param {
      EntryParam::DependencyId(dep_id, sender) => {
        if let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(&dep_id) {
          sender
            .send(Event::FinishDeps(None, dep_id, Some(*module_identifier)))
            .expect("should success");
        } else {
          // no module_identifier means the factorize task not run, do nothing
        }
        Ok(vec![])
      }
      EntryParam::EntryDependency(dep) => {
        let dep_id = *dep.id();
        module_graph.add_dependency(dep.clone());
        Ok(vec![Box::new(FactorizeTask {
          module_factory: context
            .dependency_factories
            .get(dep.dependency_type())
            .expect("should have dependency_factories")
            .clone(),
          original_module_identifier: None,
          original_module_source: None,
          issuer: None,
          original_module_context: None,
          dependency: dep,
          dependencies: vec![dep_id],
          resolve_options: None,
          options: context.compiler_options.clone(),
          current_profile: context
            .compiler_options
            .profile
            .then(Box::<ModuleProfile>::default),
        })])
      }
    }
  }
}
