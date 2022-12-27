use std::collections::VecDeque;

pub struct WorkerQueue<T> {
  inner: VecDeque<T>,
}

impl<T> WorkerQueue<T> {
  pub fn new() -> Self {
    Self {
      inner: VecDeque::new(),
    }
  }

  pub fn add_task(&self, task: T) -> usize {
    self.inner.push_back(task);
    self.inner.len()
  }

  pub fn get_task(&self) -> Option<T> {
    self.inner.pop_front()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}
