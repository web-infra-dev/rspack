use std::collections::{hash_map::Entry, VecDeque};

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::{miette::diagnostic, Error};
use rustc_hash::FxHashMap as HashMap;

use super::context::ExecutorTaskContext;
use crate::{
  make::repair::MakeTaskContext, task_loop::Task, DependencyId, ModuleIdentifier, ModuleIssuer,
};

type BoxTask = Box<dyn Task<ExecutorTaskContext>>;

/// Tracks whether a module and its submodules have been built
#[derive(Debug, Default)]
pub struct ModuleTracker {
  /// A map to record unfinished modules and its unfinished submodules count.
  ///
  /// if a module not in this map, it means the module and its submodules have already been built.
  /// if the submodules count is usize::MAX means the module is building.
  unfinished_module: IdentifierMap<usize>,

  /// Entry dependency id to target box task
  ///
  /// when entry dependency submodules are built,
  /// 1. if reused_unfinished_module is empty, return box task directly
  /// 2. if reused_unfinished_module is not empty, add box task to pending_callbacks
  entry_finish_task: HashMap<DependencyId, BoxTask>,

  /// Reused and unfinished modules
  ///
  /// When cycle use module(B -> A -> B) or multi entries use same module(A -> B, C -> B)
  /// will add the unfinished module(B) to this set.
  /// The box task can be return only when the set is empty.
  reused_unfinished_module: IdentifierSet,

  /// Pending box tasks
  ///
  /// List the box task waiting for reused_unfinished_module to be cleared.
  pending_tasks: Vec<BoxTask>,
}

impl ModuleTracker {
  fn calc_runnable_tasks(&mut self, mut new_tasks: Vec<BoxTask>) -> Vec<BoxTask> {
    if self.reused_unfinished_module.is_empty() {
      new_tasks.extend(std::mem::take(&mut self.pending_tasks));
      new_tasks
    } else {
      self.pending_tasks.extend(new_tasks);
      vec![]
    }
  }
  /// Set a module as finished. Returns runnable box tasks.
  ///
  /// This method removes the module from the tracker and
  /// recursively checks the status of all unfinished parent modules of this module.
  /// Call this method when a module and its submodules are built.
  fn finish_module(
    &mut self,
    context: &mut MakeTaskContext,
    mid: ModuleIdentifier,
  ) -> Vec<BoxTask> {
    let mut queue = VecDeque::from(vec![mid]);
    let mut tasks = vec![];
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
          // no origin module
          for dep_id in mgm.incoming_connections() {
            let connection = module_graph
              .connection_by_dependency_id(dep_id)
              .expect("should have connection");
            if let Some(task) = self.entry_finish_task.remove(&connection.dependency_id) {
              tasks.push(task);
            }
          }
        }
        ModuleIssuer::Some(mid) => {
          if let Some(count) = self.unfinished_module.get_mut(mid) {
            *count = *count - 1;
            if count == &0 {
              queue.push_back(*mid);
            }
          }
        }
      };
    }

    self.calc_runnable_tasks(tasks)
  }

  fn finish_dep(
    &mut self,
    context: &mut MakeTaskContext,
    origin_mid: Option<ModuleIdentifier>,
    dep_id: DependencyId,
  ) -> Vec<BoxTask> {
    let Some(origin_mid) = origin_mid else {
      // entry
      if let Some(task) = self.entry_finish_task.remove(&dep_id) {
        if self.reused_unfinished_module.is_empty() {
          return vec![task];
        } else {
          self.pending_tasks.push(task);
        }
      }
      return vec![];
    };
    let count = self
      .unfinished_module
      .get_mut(&origin_mid)
      .expect("should factorize parent module unfinished");
    *count = *count - 1;
    if count == &0 {
      self.finish_module(context, origin_mid)
    } else {
      vec![]
    }
  }

  pub fn on_factorize_failed(
    &mut self,
    context: &mut MakeTaskContext,
    origin_mid: Option<ModuleIdentifier>,
    dep_id: DependencyId,
  ) -> Vec<BoxTask> {
    self.finish_dep(context, origin_mid, dep_id)
  }

  pub fn on_add_resolved_module(
    &mut self,
    context: &mut MakeTaskContext,
    origin_mid: Option<ModuleIdentifier>,
    dep_id: DependencyId,
    mid: ModuleIdentifier,
  ) -> Vec<BoxTask> {
    if self.unfinished_module.contains_key(&mid) {
      self.reused_unfinished_module.insert(mid);
    }
    self.finish_dep(context, origin_mid, dep_id)
  }

  pub fn on_add(&mut self, mid: ModuleIdentifier) {
    // will build module
    self.unfinished_module.insert(mid, usize::MAX);
  }

  pub fn on_process_dependencies(
    &mut self,
    context: &mut MakeTaskContext,
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

  pub fn on_entry(
    &mut self,
    context: &mut MakeTaskContext,
    dep_id: DependencyId,
    get_task: impl FnOnce(Option<Error>) -> Option<BoxTask>,
  ) -> Vec<BoxTask> {
    let Entry::Vacant(entry) = self.entry_finish_task.entry(dep_id) else {
      // entry task already exist means have circular build dependency
      let _ = get_task(Some(
        diagnostic!(
          "task exist for {:?}, maybe have a circular build dependency",
          dep_id
        )
        .into(),
      ));
      return vec![];
    };

    let Some(task) = get_task(None) else {
      return vec![];
    };
    let mg = context.artifact.get_module_graph();
    if let Some(mid) = mg.module_identifier_by_dependency_id(&dep_id) {
      if self.unfinished_module.contains_key(&mid) {
        // The target module is unfinished, add reuse and pending.
        self.reused_unfinished_module.insert(*mid);
        self.pending_tasks.push(task);
        vec![]
      } else {
        // The target module is complete and the task can be run.
        self.calc_runnable_tasks(vec![task])
      }
    } else {
      // The module corresponding to the entry does not exist,
      // just insert it and wait for it to factorize.
      entry.insert(task);
      vec![]
    }
  }
}
