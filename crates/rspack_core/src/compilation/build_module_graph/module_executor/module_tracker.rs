use std::collections::{VecDeque, hash_map::Entry};

use rspack_collections::{IdentifierMap, IdentifierSet};
use rustc_hash::FxHashMap as HashMap;

use super::{super::graph_updater::repair::context::TaskContext, context::ExecutorTaskContext};
use crate::{DependencyId, ModuleIdentifier, ModuleIssuer, task_loop::Task};

type BoxTask = Box<dyn Task<ExecutorTaskContext>>;

/// Tracks whether a module and its submodules have been built.
#[derive(Debug, Default)]
pub struct ModuleTracker {
  /// A map to record unfinished modules and its unfinished submodules count.
  ///
  /// If a module not in this map, it means the module and its submodules have already been built.
  /// If the submodules count is usize::MAX means the module is building.
  unfinished_module: IdentifierMap<usize>,

  /// Entry dependency id to target box tasks.
  ///
  /// when entry dependency submodules are built,
  /// 1. if reused_unfinished_module is empty, return box tasks directly.
  /// 2. if reused_unfinished_module is not empty, add box tasks to pending_callbacks.
  entry_finish_tasks: HashMap<DependencyId, Vec<BoxTask>>,

  /// Reused and unfinished modules.
  ///
  /// When cycle use module(B -> A -> B) or multi entries use same module(A -> B, C -> B)
  /// will add the unfinished module(B) to this set.
  /// The box task can be return only when the set is empty.
  reused_unfinished_module: IdentifierSet,

  /// Pending box tasks.
  ///
  /// List the box task waiting for reused_unfinished_module to be cleared.
  pending_tasks: Vec<BoxTask>,
}

impl ModuleTracker {
  /// Calculate runnable tasks.
  ///
  /// This method will determine whether the ready_tasks and self.pending tasks are runnable.
  /// Return all tasks only if self.reused_unfinished_module is empty, otherwise leave ready tasks pending.
  fn calc_runnable_tasks(&mut self, mut ready_tasks: Vec<BoxTask>) -> Vec<BoxTask> {
    if self.reused_unfinished_module.is_empty() {
      ready_tasks.extend(std::mem::take(&mut self.pending_tasks));
      ready_tasks
    } else {
      self.pending_tasks.extend(ready_tasks);
      vec![]
    }
  }

  /// Set a module as finished. Returns runnable box tasks.
  ///
  /// This method removes the module from the tracker and
  /// recursively checks the status of all unfinished parent modules of this module.
  /// Call this method when a module and its submodules are built.
  fn finish_module(&mut self, context: &mut TaskContext, mid: ModuleIdentifier) -> Vec<BoxTask> {
    let mut queue = VecDeque::from(vec![mid]);
    let mut ready_tasks = vec![];
    let module_graph = context.artifact.get_module_graph();
    while let Some(module_identifier) = queue.pop_front() {
      tracing::debug!("finish build module {:?}", module_identifier);
      let count = self.unfinished_module.remove(&module_identifier);
      debug_assert_eq!(count, Some(0));

      self.reused_unfinished_module.remove(&module_identifier);

      let mgm = module_graph
        .module_graph_module_by_identifier(&module_identifier)
        .expect("should have mgm");

      match mgm.issuer() {
        ModuleIssuer::Unset => {
          panic!("can not access unset module issuer");
        }
        ModuleIssuer::None => {
          // no origin module, module is a entry module
          for dep_id in mgm.incoming_connections() {
            let connection = module_graph
              .connection_by_dependency_id(dep_id)
              .expect("should have connection");
            if let Some(tasks) = self.entry_finish_tasks.remove(&connection.dependency_id) {
              ready_tasks.extend(tasks);
            }
          }
        }
        ModuleIssuer::Some(mid) => {
          if let Some(count) = self.unfinished_module.get_mut(mid) {
            *count -= 1;
            if count == &0 {
              queue.push_back(*mid);
            }
          }
        }
      };
    }

    self.calc_runnable_tasks(ready_tasks)
  }

  /// Set a dependency as finished. Returns runnable box tasks.
  ///
  /// If origin_mid is None, the method directly returns the runnable box task.
  /// If origin_mid exists, origin_mid's unfinished submodules will be minus one,
  /// and trigger self.finish_module if submodules count becomes 0.
  /// Call this method when a dependency processing is complete.
  fn finish_dep(
    &mut self,
    context: &mut TaskContext,
    origin_mid: Option<ModuleIdentifier>,
    dep_id: DependencyId,
  ) -> Vec<BoxTask> {
    let Some(origin_mid) = origin_mid else {
      // entry
      if let Some(tasks) = self.entry_finish_tasks.remove(&dep_id) {
        return self.calc_runnable_tasks(tasks);
      }
      return vec![];
    };
    let count = self
      .unfinished_module
      .get_mut(&origin_mid)
      .expect("should factorize parent module unfinished");
    *count -= 1;
    if count == &0 {
      self.finish_module(context, origin_mid)
    } else {
      vec![]
    }
  }

  /// Handle factorize task failed.
  pub fn on_factorize_failed(
    &mut self,
    context: &mut TaskContext,
    origin_mid: Option<ModuleIdentifier>,
    dep_id: DependencyId,
  ) -> Vec<BoxTask> {
    self.finish_dep(context, origin_mid, dep_id)
  }

  /// Handle add task with resolved module.
  pub fn on_add_resolved_module(
    &mut self,
    context: &mut TaskContext,
    origin_mid: Option<ModuleIdentifier>,
    dep_id: DependencyId,
    mid: ModuleIdentifier,
  ) -> Vec<BoxTask> {
    if self.unfinished_module.contains_key(&mid) {
      self.reused_unfinished_module.insert(mid);
    }
    self.finish_dep(context, origin_mid, dep_id)
  }

  /// Handle add task with module need build
  pub fn on_add(&mut self, mid: ModuleIdentifier) {
    // will build module
    self.unfinished_module.insert(mid, usize::MAX);
  }

  /// Handle process dependencies task.
  pub fn on_process_dependencies(
    &mut self,
    context: &mut TaskContext,
    mid: ModuleIdentifier,
    child_count: usize,
  ) -> Vec<BoxTask> {
    self.unfinished_module.insert(mid, child_count);
    if child_count == 0 {
      self.finish_module(context, mid)
    } else {
      vec![]
    }
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
          self.calc_runnable_tasks(vec![task])
        }
      }
    }
  }

  /// Check if a dep_id is running
  pub fn is_running(&self, dep_id: DependencyId) -> bool {
    self.entry_finish_tasks.contains_key(&dep_id)
  }
}
