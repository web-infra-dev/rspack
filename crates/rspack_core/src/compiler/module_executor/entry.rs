use rspack_collections::Identifier;
use tokio::sync::mpsc::UnboundedSender;

use super::{ctrl::Event, overwrite::OverwriteTask};
use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Context, Dependency, DependencyId, LoaderImportDependency, Module, ModuleProfile,
  NormalModuleSource,
};

#[derive(Debug)]
pub enum EntryParam {
  DependencyId(DependencyId),
  Entry {
    original_module_context: Option<Context>,
    original_module_identifier: Option<Identifier>,
    dep: Box<LoaderImportDependency>,
  },
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
      EntryParam::Entry {
        original_module_identifier,
        original_module_context,
        dep,
      } => {
        module_graph.add_dependency(dep.clone());
        let module = original_module_identifier
          .as_ref()
          .and_then(|original_module_identifier| {
            module_graph.module_by_identifier(original_module_identifier)
          })
          .and_then(|module| module.as_normal_module());
        let original_module_source = module.as_ref().and_then(|module| {
          if let NormalModuleSource::BuiltSucceed(s) = module.source() {
            Some(s.clone())
          } else {
            None
          }
        });
        let resolve_options = module
          .as_ref()
          .and_then(|module| module.get_resolve_options());

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
          original_module_identifier,
          original_module_source,
          issuer: module
            .as_ref()
            .and_then(|module| module.name_for_condition()),
          issuer_layer: module
            .as_ref()
            .and_then(|module| module.get_layer())
            .map(|layer| layer.clone()),
          original_module_context: original_module_context.map(|ctx| Box::new(ctx)),
          dependencies: vec![dep],
          resolve_options,
          options: context.compiler_options.clone(),
          current_profile: context
            .compiler_options
            .profile
            .then(Box::<ModuleProfile>::default),
          recursive: true,
          connect_origin: false,
        });
        Ok(vec![Box::new(OverwriteTask {
          origin_task: task,
          event_sender,
        })])
      }
    }
  }
}
