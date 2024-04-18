use super::MakeTaskContext;
use crate::{
  utils::task_loop::{Task, TaskResult, TaskType},
  ModuleIdentifier,
};

pub struct CleanTask {
  pub module_identifier: ModuleIdentifier,
}

impl Task<MakeTaskContext> for CleanTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }
  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let module_identifier = self.module_identifier;
    let module_graph = &mut MakeTaskContext::get_module_graph(&mut context.module_graph_partial);
    let Some(mgm) = module_graph.module_graph_module_by_identifier(&module_identifier) else {
      tracing::trace!("Module is cleaned: {}", module_identifier);
      return Ok(vec![]);
    };

    if !mgm.incoming_connections().is_empty() {
      tracing::trace!("Module is used: {}", module_identifier);
      return Ok(vec![]);
    }

    let dependent_module_identifiers: Vec<ModuleIdentifier> = module_graph
      .get_module_all_depended_modules(&module_identifier)
      .expect("should have module")
      .into_iter()
      .copied()
      .collect();
    module_graph.revoke_module(&module_identifier);

    let mut res: Vec<Box<dyn Task<MakeTaskContext>>> =
      Vec::with_capacity(dependent_module_identifiers.len());
    for module_identifier in dependent_module_identifiers {
      res.push(Box::new(CleanTask { module_identifier }))
    }
    Ok(res)
  }
}
