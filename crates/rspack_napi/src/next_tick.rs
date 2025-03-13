use std::sync::LazyLock;

use crossbeam_queue::SegQueue;

pub(crate) static QUEUE: LazyLock<SegQueue<Box<dyn FnOnce() + Send + 'static>>> =
  LazyLock::new(|| SegQueue::new());

pub fn next_tick<T: FnOnce() + Send + 'static>(f: T) {
  let f = Box::new(f);
  QUEUE.push(f);
}
