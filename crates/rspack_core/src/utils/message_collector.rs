use std::sync::Mutex;

use crossbeam_channel::{unbounded, Iter, Receiver, Sender};

#[derive(Debug)]
pub struct MessageCollector<T> {
  tx: Mutex<Option<Sender<T>>>,
  rx: Receiver<T>,
}

impl<T> Default for MessageCollector<T> {
  fn default() -> Self {
    let (tx, rx) = unbounded();
    Self {
      tx: Mutex::new(Some(tx)),
      rx,
    }
  }
}

impl<T> MessageCollector<T> {
  pub fn iter(&self) -> Iter<'_, T> {
    let mut guard = self.tx.lock().expect("lock for MessageCollector failed");
    let tx = guard.take();
    drop(tx);
    self.rx.iter()
  }

  pub fn sender(&self) -> Option<Sender<T>> {
    self
      .tx
      .lock()
      .expect("lock for MessageCollector failed")
      .clone()
  }
}
