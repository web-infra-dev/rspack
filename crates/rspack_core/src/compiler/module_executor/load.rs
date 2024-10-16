use std::path::PathBuf;

use rspack_sources::{MapOptions, SourceMap};
use rustc_hash::FxHashSet as HashSet;
use tokio::sync::oneshot::Sender;

use crate::{
  compiler::make::repair::MakeTaskContext,
  task_loop::{Task, TaskResult, TaskType},
  DependencyId,
};

#[derive(Debug)]
pub struct LoadModuleResult {
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub source: Option<String>,
  pub map: Option<SourceMap>,
}

#[derive(Debug)]
pub struct LoadTask {
  pub entry_dep_id: DependencyId,
  pub result_sender: Sender<LoadModuleResult>,
}

impl Task<MakeTaskContext> for LoadTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self {
      entry_dep_id,
      result_sender,
    } = *self;

    let mg = MakeTaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);

    let module = mg
      .get_module_by_dependency_id(&entry_dep_id)
      .expect("should have module");

    let build_info = module.build_info();

    result_sender
      .send(LoadModuleResult {
        file_dependencies: build_info
          .map_or_else(HashSet::default, |info| info.file_dependencies.clone()),
        context_dependencies: build_info
          .map_or_else(HashSet::default, |info| info.context_dependencies.clone()),
        missing_dependencies: build_info
          .map_or_else(HashSet::default, |info| info.missing_dependencies.clone()),
        build_dependencies: build_info
          .map_or_else(HashSet::default, |info| info.build_dependencies.clone()),
        source: module
          .original_source()
          .map(|source| source.source().to_string()),
        map: module
          .original_source()
          .and_then(|source| source.map(&MapOptions::default())),
      })
      .expect("should send result success");
    Ok(vec![])
  }
}
