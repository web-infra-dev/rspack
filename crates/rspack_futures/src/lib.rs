use std::{
  future::Future,
  ops::{Deref, DerefMut},
};

/// Run futures in parallel.
///
///
/// # Panic
///
/// Panics if any task panics.
///
/// A rough demo of how this works:
///
/// # Example
///
/// ```rust,ignore
/// use rspack_futures::FuturesResults;
///
/// #[tokio::main]
/// fn main() {
///   let futures = vec![async { 1 }, async { 2 }];
///   let results = futures.into_iter().collect::<FuturesResults>();
///
///   assert_eq!(results, vec![Ok(1), Ok(2)]);
/// }
/// ```
#[derive(Default)]
pub struct FuturesResults<T> {
  inner: Vec<T>,
}

impl<T> FuturesResults<T> {
  pub fn into_inner(self) -> Vec<T> {
    self.inner
  }
}

#[cfg(not(target_family = "wasm"))]
impl<Fut> FromIterator<Fut> for FuturesResults<Fut::Output>
where
  Fut: Future + Send,
  Fut::Output: Send + 'static,
{
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = Fut>,
  {
    use async_scoped::{Scope, TokioScope};

    let (_, inner) = Scope::scope_and_block(|s: &mut TokioScope<'_, _>| {
      iter.into_iter().for_each(|fut| {
        s.spawn(fut);
      });
    });

    Self {
      inner: inner
        .into_iter()
        .map(|i| match i {
          Ok(i) => i,
          Err(err) => {
            if err.is_panic() {
              std::panic::resume_unwind(err.into_panic())
            } else {
              unreachable!("Error should be a panic {err}")
            }
          }
        })
        .collect(),
    }
  }
}

#[cfg(target_family = "wasm")]
impl<Fut> FromIterator<Fut> for FuturesResults<Fut::Output>
where
  Fut: Future + Send,
  Fut::Output: Send + 'static,
{
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = Fut>,
  {
    use futures::future::join_all;

    Self {
      inner: futures::executor::block_on(join_all(iter)),
    }
  }
}

impl<T> Deref for FuturesResults<T> {
  type Target = Vec<T>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T> DerefMut for FuturesResults<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}
