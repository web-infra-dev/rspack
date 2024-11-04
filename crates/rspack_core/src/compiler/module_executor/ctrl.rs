use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::sync::mpsc::UnboundedReceiver;

use super::{entry::EntryTask, execute::ExecuteTask};
use crate::{
  compiler::make::repair::MakeTaskContext,
  utils::task_loop::{Task, TaskResult, TaskType},
  Dependency, DependencyId, LoaderImportDependency, ModuleIdentifier,
};

#[derive(Debug, Default)]
struct UnfinishCounter {
  is_building: bool,
  unfinished_child_module_count: usize,
}

impl UnfinishCounter {
  fn set_children(&mut self, size: usize) {
    self.is_building = false;
    self.unfinished_child_module_count = size;
  }

  fn minus_one(&mut self) -> bool {
    if self.is_building || self.unfinished_child_module_count == 0 {
      panic!("UnfinishDepCount Error")
    }
    self.unfinished_child_module_count -= 1;

    self.is_finished()
  }

  fn is_finished(&self) -> bool {
    !self.is_building && self.unfinished_child_module_count == 0
  }
}

#[derive(Debug, Default)]
struct ExecuteTaskList(Vec<Box<dyn Task<MakeTaskContext>>>);

impl ExecuteTaskList {
  fn add_task(&mut self, task: ExecuteTask) {
    self.0.push(Box::new(task));
    if self.0.len() > 10000 {
      // TODO change to Err
      panic!("ExecuteTaskList exceeds limit and may contain circular build dependencies.")
    }
  }

  fn into_vec(self) -> Vec<Box<dyn Task<MakeTaskContext>>> {
    self.0
  }
}

#[derive(Debug)]
pub enum ExecuteParam {
  DependencyId(DependencyId),
  Entry(Box<LoaderImportDependency>, Option<String>),
}

// send event can only use in sync task
#[derive(Debug)]
pub enum Event {
  Add(
    Option<ModuleIdentifier>,
    ModuleIdentifier,
    DependencyId,
    bool,
  ),
  ProcessDeps(ModuleIdentifier, usize),
  FinishModule(ModuleIdentifier),
  ExecuteModule(ExecuteParam, ExecuteTask),
  Stop,
}

#[derive(Debug)]
pub struct CtrlTask {
  pub event_receiver: UnboundedReceiver<Event>,
  execute_task_map: HashMap<DependencyId, ExecuteTaskList>,
  running_module_map: HashMap<ModuleIdentifier, UnfinishCounter>,
  imported_module: HashMap<ModuleIdentifier, DependencyId>,
  finished_modules: HashSet<ModuleIdentifier>,
  module_deps: HashMap<ModuleIdentifier, HashSet<ModuleIdentifier>>,
}

impl CtrlTask {
  pub fn new(event_receiver: UnboundedReceiver<Event>) -> Self {
    Self {
      event_receiver,
      execute_task_map: Default::default(),
      running_module_map: Default::default(),
      imported_module: Default::default(),
      finished_modules: Default::default(),
      module_deps: Default::default(),
    }
  }

  fn try_finish(
    &mut self,
    module: ModuleIdentifier,
  ) -> Option<Vec<Box<dyn Task<MakeTaskContext>>>> {
    // finish this module and its importers if could
    self.finish_module(module);

    // check if we are ready to execute
    let mut res: Vec<Box<dyn Task<MakeTaskContext>>> = vec![];
    for (m, dep_id) in &self.imported_module {
      if self.finished_modules.contains(m)
        && let Some(tasks) = self.execute_task_map.remove(dep_id)
      {
        res.extend(tasks.into_vec());
      }
    }

    if !res.is_empty() {
      Some(res)
    } else {
      None
    }
  }

  fn finish_module(&mut self, module_identifier: ModuleIdentifier) {
    let finished = !self.finished_modules.insert(module_identifier);
    if finished {
      return;
    }

    // finish all incomings recursively
    let mut to_be_finished: Vec<rspack_collections::Identifier> = vec![];

    if let Some(importers) = self.module_deps.get(&module_identifier) {
      for importer in importers {
        let importer = *importer;

        if let Some(counter) = self.running_module_map.get_mut(&importer) {
          let finished = counter.minus_one();
          if finished {
            to_be_finished.push(importer);
          }
        }
      }
    }

    for importer in to_be_finished {
      self.finish_module(importer);
    }
  }
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for CtrlTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Async
  }

  async fn async_run(mut self: Box<Self>) -> TaskResult<MakeTaskContext> {
    while let Some(event) = self.event_receiver.recv().await {
      tracing::info!("CtrlTask async receive {:?}", event);
      match event {
        Event::Add(orig_module, target_module, dep_id, is_self_module) => {
          if let Some(orig_module) = orig_module {
            self
              .module_deps
              .entry(target_module)
              .or_default()
              .insert(orig_module);

            if is_self_module {
              let counter = self.running_module_map.entry(orig_module).or_default();
              let finish = counter.minus_one();
              if finish && let Some(mut tasks) = self.try_finish(orig_module) {
                tasks.push(self);
                return Ok(tasks);
              }
            }
          } else {
            self.imported_module.insert(target_module, dep_id);
          }
        }
        Event::ProcessDeps(module, deps) => {
          self
            .running_module_map
            .entry(module)
            .or_default()
            .set_children(deps);
        }
        Event::FinishModule(module) => {
          // finish this module and its importers if could
          if let Some(mut tasks) = self.try_finish(module) {
            tasks.push(self);
            return Ok(tasks);
          }
        }
        Event::ExecuteModule(param, execute_task) => {
          match param {
            ExecuteParam::Entry(dep, layer) => {
              // user call importModule the first time
              let dep_id = dep.id();
              if let Some(tasks) = self.execute_task_map.get_mut(dep_id) {
                // already compiled this entry, but not finished
                tasks.add_task(execute_task)
              } else {
                // setup compile
                let mut list = ExecuteTaskList::default();
                list.add_task(execute_task);
                self.execute_task_map.insert(*dep_id, list);
                return Ok(vec![Box::new(EntryTask { dep, layer }), self]);
              }
            }
            ExecuteParam::DependencyId(dep_id) => {
              if let Some(tasks) = self.execute_task_map.get_mut(&dep_id) {
                tasks.add_task(execute_task)
              } else {
                return Ok(vec![Box::new(execute_task), self]);
              }
            }
          };
        }
        Event::Stop => {
          return Ok(vec![]);
        }
      }
    }
    // if channel has been closed, finish this task
    Ok(vec![])
  }
}
