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
  use rayon::prelude::*;

  use super::RayonConsumer;

  #[test]
  fn available() {
    (0..10)
      .into_par_iter()
      .map(|item| item * 2)
      .consume(|item| assert_eq!(item % 2, 0));
  }
}
