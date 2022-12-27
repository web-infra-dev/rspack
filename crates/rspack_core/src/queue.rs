use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
};

pub struct WorkerQueue<T> {
  inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send> WorkerQueue<T> {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(VecDeque::new())),
    }
  }

  pub fn add_task(&self, task: T) -> usize {
    let mut lock = self.inner.lock().unwrap();
    lock.push_back(task);
    lock.len()
  }

  pub fn get_task(&self) -> Option<T> {
    self.inner.lock().unwrap().pop_front()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.lock().unwrap().is_empty()
  }
}
