use std::collections::VecDeque;

use rustc_hash::FxHashMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::{Compilation, ModuleIdentifier};

pub struct WorkerQueue<T, K> {
  inner: VecDeque<T>,
  queue_tx: QueueHandler<T, K>,
  queue_rx: UnboundedReceiver<TaskItem<T, K>>,
  finished: FxHashMap<K, ModuleIdentifier>,
  waiting: FxHashMap<K, Vec<QueueHandleCallback>>,
}

impl<T, K: Eq + PartialEq + std::hash::Hash> Default for WorkerQueue<T, K> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T, K: Eq + PartialEq + std::hash::Hash> WorkerQueue<T, K> {
  pub fn new() -> Self {
    let (queue_tx, queue_rx) = unbounded_channel();

    Self {
      inner: VecDeque::new(),
      queue_tx: QueueHandler { inner: queue_tx },
      queue_rx,
      finished: Default::default(),
      waiting: Default::default(),
    }
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub fn add_task(&mut self, task: T) -> usize {
    self.inner.push_back(task);
    self.inner.len()
  }

  pub fn add_tasks(&mut self, tasks: impl IntoIterator<Item = T>) -> usize {
    self.inner.extend(tasks);
    self.inner.len()
  }

  pub fn get_task(&mut self, compilation: &mut Compilation) -> Option<T> {
    self.try_process(compilation);
    self.inner.pop_front()
  }

  pub fn queue_handler(&self) -> QueueHandler<T, K> {
    self.queue_tx.clone()
  }

  pub fn complete_task(
    &mut self,
    key: K,
    module_identifier: ModuleIdentifier,
    compilation: &mut Compilation,
  ) {
    if let Some(callbacks) = self.waiting.get_mut(&key) {
      while let Some(callback) = callbacks.pop() {
        callback(module_identifier, compilation);
      }
    }
    self.finished.insert(key, module_identifier);
  }

  pub fn try_process(&mut self, compilation: &mut Compilation) {
    while let Ok(task) = self.queue_rx.try_recv() {
      match task {
        TaskItem::Task(task) => self.inner.push_back(task),
        TaskItem::Wait(key, callback) => {
          if let Some(result) = self.finished.get(&key) {
            callback(*result, compilation);
          } else {
            let wait_list = self.waiting.entry(key).or_default();
            wait_list.push(callback);
          }
        }
      }
    }
  }
}

#[derive(Debug)]
pub struct QueueHandler<T, K> {
  inner: UnboundedSender<TaskItem<T, K>>,
}

impl<T, K> Clone for QueueHandler<T, K> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

enum TaskItem<T, K> {
  Task(T),
  Wait(K, QueueHandleCallback),
}

pub type QueueHandleCallback = Box<dyn FnOnce(ModuleIdentifier, &mut Compilation) + Send + Sync>;

impl<T, K> QueueHandler<T, K> {
  pub fn add_task(&self, task: T) {
    self
      .inner
      .send(TaskItem::Task(task))
      .expect("failed to send channel message");
  }

  pub fn wait_for(&self, key: K, callback: QueueHandleCallback) {
    self
      .inner
      .send(TaskItem::Wait(key, callback))
      .expect("failed to send channel message");
  }
}
