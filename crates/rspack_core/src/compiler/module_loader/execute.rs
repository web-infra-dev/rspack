use rspack_error::{miette::miette, Error};

use super::context::{Callback, LoadModuleMeta, LoadTaskContext};
use crate::{
  utils::task_loop::{Task, TaskResult, TaskType},
  FactorizeInfo,
};

#[derive(Debug)]
pub struct ExecuteTask {
  pub meta: LoadModuleMeta,
  pub callback: Callback,
}

impl ExecuteTask {
  pub fn finish_with_error(self, error: Error) {
    self.callback.0(Err(error));
  }
}

#[async_trait::async_trait]
impl Task<LoadTaskContext> for ExecuteTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut LoadTaskContext) -> TaskResult<LoadTaskContext> {
    let Self { meta, callback } = *self;

    let LoadTaskContext {
      origin_context,
      entries,
      ..
    } = context;

    let entry_dep_id = entries.get(&meta).expect("should have dep_id");
    let mg = origin_context.artifact.get_module_graph();
    if origin_context
      .artifact
      .make_failed_dependencies
      .contains(entry_dep_id)
    {
      // factorize failed
      let entry_dep = mg.dependency_by_id(entry_dep_id).expect("should have dep");
      let info = FactorizeInfo::get_from(entry_dep).expect("should have factorize info");
      let error = info
        .diagnostics()
        .first()
        .expect("should have error")
        .clone();
      callback.0(Err(miette!(error.to_string())));
      return Ok(vec![]);
    }

    let module = mg
      .get_module_by_dependency_id(entry_dep_id)
      .expect("should module exist");
    callback.0(Ok(module));
    Ok(vec![])
  }
}
