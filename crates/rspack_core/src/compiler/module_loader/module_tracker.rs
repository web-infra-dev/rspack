use std::collections::hash_map::Entry;

use rustc_hash::FxHashMap as HashMap;

use super::context::LoadTaskContext;
use crate::{make::repair::MakeTaskContext, task_loop::Task, DependencyId, ModuleIdentifier};

type BoxTask = Box<dyn Task<LoadTaskContext>>;

/// Tracks whether a module and its submodules have been built.
#[derive(Debug, Default)]
pub struct ModuleTracker {
  /// Entry dependency id to target box tasks.
  entry_finish_tasks: HashMap<DependencyId, Vec<BoxTask>>,
}

impl ModuleTracker {
  /// Set a dependency as finished. Returns runnable box tasks.
  fn finish_dep(&mut self, dep_id: &DependencyId) -> Vec<BoxTask> {
    self.entry_finish_tasks.remove(dep_id).unwrap_or_default()
  }

  /// Handle factorize task failed.
  pub fn on_factorize_failed(&mut self, dep_id: &DependencyId) -> Vec<BoxTask> {
    self.finish_dep(dep_id)
  }

  /// Handle add task with resolved module.
  pub fn on_add_resolved_module(&mut self, dep_id: &DependencyId) -> Vec<BoxTask> {
    self.finish_dep(dep_id)
  }

  /// Handle build result task.
  pub fn on_build_result(
    &mut self,
    context: &mut MakeTaskContext,
    module_id: &ModuleIdentifier,
  ) -> Vec<BoxTask> {
    let mut res = vec![];
    let mg = context.artifact.get_module_graph();
    for connect in mg.get_incoming_connections(module_id) {
      if let Some(tasks) = self.entry_finish_tasks.remove(&connect.dependency_id) {
        res.extend(tasks);
      }
    }
    res
  }

  /// Handle entry task.
  pub fn on_entry(&mut self, is_new: bool, dep_id: DependencyId, task: BoxTask) -> Vec<BoxTask> {
    match self.entry_finish_tasks.entry(dep_id) {
      Entry::Occupied(mut entry) => {
        entry.get_mut().push(task);
        vec![]
      }
      Entry::Vacant(entry) => {
        if is_new {
          // insert it and wait for it to factorize.
          entry.insert(vec![task]);
          vec![]
        } else {
          // The target module is complete and the task can be run.
          vec![task]
        }
      }
    }
  }

  /// Check if a dep_id is running
  pub fn is_running(&self, dep_id: &DependencyId) -> bool {
    self.entry_finish_tasks.contains_key(dep_id)
  }
}
