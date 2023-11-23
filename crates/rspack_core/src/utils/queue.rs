use std::any::Any;
use std::collections::VecDeque;
use std::hash::Hash;

use rustc_hash::FxHashMap;
use tokio::sync::mpsc::UnboundedSender;

pub trait Task<Key> {
  fn get_key(&self) -> Key;
}

impl<T> Task<()> for T
where
  T: Any,
{
  fn get_key(&self) {}
}

struct QueueEntry<Key> {
  finish: bool,
  callbacks: Vec<UnboundedSender<Key>>,
}

impl<Key> std::fmt::Debug for QueueEntry<Key> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("QueueEntry")
      .field("finish", &self.finish)
      .finish()
  }
}

impl<Key> QueueEntry<Key> {
  fn new() -> Self {
    Self {
      finish: false,
      callbacks: Default::default(),
    }
  }
}

#[derive(Default, Debug)]
pub struct WorkerQueue<T: Task<Key>, Key: std::fmt::Debug + Eq + Hash = ()> {
  inner: VecDeque<T>,
  entries: FxHashMap<Key, QueueEntry<Key>>,
}

#[allow(clippy::unwrap_in_result)]
impl<T: Task<Key>, Key: Hash + Clone + Eq + std::fmt::Debug> WorkerQueue<T, Key> {
  pub fn new() -> Self {
    Self {
      inner: VecDeque::new(),
      entries: Default::default(),
    }
  }

  pub fn add_task(&mut self, task: T) -> usize {
    self.entries.insert(task.get_key(), QueueEntry::new());
    self.inner.push_back(task);
    self.inner.len()
  }

  pub fn add_task_with_callback(&mut self, task: T, callback: UnboundedSender<Key>) -> usize {
    let entry = QueueEntry {
      callbacks: vec![callback],
      finish: false,
    };

    self.entries.insert(task.get_key(), entry);
    self.inner.push_back(task);
    self.inner.len()
  }

  pub fn add_tasks(&mut self, tasks: impl IntoIterator<Item = T>) -> usize {
    tasks.into_iter().for_each(|task| {
      self.add_task(task);
    });
    self.inner.len()
  }

  pub fn get_task(&mut self) -> Option<T> {
    if let Some(inner) = self.inner.pop_front() {
      let key = inner.get_key();
      if let Some(entry) = self.entries.get_mut(&key) {
        entry.finish = true;
        while let Some(callback) = entry.callbacks.pop() {
          callback
            .send(key.clone())
            .expect("Failed to notify task result");
        }
      }

      Some(inner)
    } else {
      None
    }
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub fn wait_for(&mut self, key: Key, callback: UnboundedSender<Key>) {
    if let Some(entry) = self.entries.get_mut(&key) {
      if entry.finish {
        callback.send(key).expect("Failed to notify task result");
      } else {
        entry.callbacks.push(callback);
      }
    }
  }
}
