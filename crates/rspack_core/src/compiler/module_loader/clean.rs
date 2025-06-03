use rspack_collections::IdentifierSet;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use super::context::LoadTaskContext;
use crate::{
  utils::task_loop::{Task, TaskResult, TaskType},
  FactorizeInfo, ModuleGraph,
};

#[derive(Debug)]
pub struct CleanModuleTask {
  pub changed_files: HashSet<ArcPath>,
}

#[async_trait::async_trait]
impl Task<LoadTaskContext> for CleanModuleTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut LoadTaskContext) -> TaskResult<LoadTaskContext> {
    let artifact = &mut context.origin_context.artifact;
    let mut mg = ModuleGraph::new(vec![], Some(&mut artifact.module_graph_partial));
    let mut affected_module = vec![];
    for (mid, module) in mg.modules() {
      if module.need_build() || module.depends_on(&self.changed_files) {
        affected_module.push(mid);
      }
    }
    let mut affect_deps = vec![];
    for mid in affected_module {
      for (dep_id, _) in mg.revoke_module(&mid) {
        mg.revoke_dependency(&dep_id, true);
        affect_deps.push(dep_id);
      }
    }

    // check dependency factorize failed
    for dep_id in &artifact.make_failed_dependencies {
      let dep = mg.dependency_by_id(dep_id).expect("should have dependency");
      let info = FactorizeInfo::get_from(dep).expect("should have factorize info");
      if info.depends_on(&self.changed_files) {
        affect_deps.push(*dep_id);
      }
    }
    context.entries.retain(|_k, v| !affect_deps.contains(v));
    Ok(vec![])
  }
}

#[derive(Debug)]
pub struct CleanEntryTask {
  pub revoked_module: IdentifierSet,
}

#[async_trait::async_trait]
impl Task<LoadTaskContext> for CleanEntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut LoadTaskContext) -> TaskResult<LoadTaskContext> {
    let revoked_module = context
      .origin_context
      .artifact
      .revoked_modules
      .iter()
      .chain(self.revoked_module.iter())
      .collect::<HashSet<_>>();
    context.entries.retain(|k, v| {
      !revoked_module.contains(&k.origin_module_identifier) || context.used_entry.contains(v)
    });
    Ok(vec![])
  }
}
