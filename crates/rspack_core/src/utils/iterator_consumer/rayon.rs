use std::sync::mpsc::channel;

use rayon::iter::ParallelIterator;

/// Tools for consume rayon iterator.
pub trait RayonConsumer {
  type Item: Send;
  /// Use to immediately consume the data produced by the rayon iterator
  /// without waiting for all the data to be processed.
  /// The closures runs in the current thread.
  fn consume(self, func: impl FnMut(Self::Item));
}

impl<P, I> RayonConsumer for P
where
  P: ParallelIterator<Item = I>,
  I: Send,
{
  type Item = I;

  #[cfg(not(target_family = "wasm"))]
  fn consume(self, mut func: impl FnMut(Self::Item)) {
    let (tx, rx) = channel::<Self::Item>();
    std::thread::scope(|s| {
      // move rx to s.spawn, otherwise rx.into_iter will never stop
      s.spawn(move || {
        self.for_each(|item| tx.send(item).expect("should send success"));
      });
      while let Ok(data) = rx.recv() {
        func(data);
      }
    });
  }

  #[cfg(target_family = "wasm")]
  fn consume(self, mut func: impl FnMut(Self::Item)) {
    let items: Vec<Self::Item> = self.collect();
    for item in items {
      func(item)
    }
  }
}

#[cfg(test)]
mod test {
  use std::time::{Duration, SystemTime};

  use rayon::prelude::*;

  use super::RayonConsumer;

  #[test]
  fn available() {
    (0..10)
      .into_par_iter()
      .map(|item| item * 2)
      .consume(|item| assert_eq!(item % 2, 0));
  }

  #[test]
  fn time_check() {
    let start = SystemTime::now();
    vec![100, 200]
      .into_par_iter()
      .map(|item| {
        std::thread::sleep(Duration::from_millis(item));
        item
      })
      .consume(|_| {
        std::thread::sleep(Duration::from_millis(20));
      });
    let time1 = SystemTime::now().duration_since(start).unwrap();

    let start = SystemTime::now();
    let data: Vec<_> = vec![100, 200]
      .into_par_iter()
      .map(|item| {
        std::thread::sleep(Duration::from_millis(item));
        item
      })
      .collect();
    for _ in data {
      std::thread::sleep(Duration::from_millis(20));
    }
    let time2 = SystemTime::now().duration_since(start).unwrap();
    assert!(time1 < time2);
  }
}
