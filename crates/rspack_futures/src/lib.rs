use std::{
  future::Future,
  ops::{Deref, DerefMut},
};

use async_scoped::{Scope, TokioScope};
use tokio::task::JoinError;

type FuturesResult<T> = Vec<Result<T, JoinError>>;

/// A set of futures that are spawned and executed in parallel and the results are returned in the order they were.
///
/// A rough demo of how this works:
///
/// # Example
///
/// ```rust,ignore
/// use rspack_futures::FuturesResults;
///
/// fn main() {
///   let futures = vec![async { 1 }, async { 2 }];
///   let results = futures.into_iter().collect::<FuturesResults>();
///
///   assert_eq!(results, vec![Ok(1), Ok(2)]
/// }
/// ```
#[derive(Default)]
pub struct FuturesResults<T> {
  inner: FuturesResult<T>,
}

impl<T> FuturesResults<T> {
  pub fn into_inner(self) -> FuturesResult<T> {
    self.inner
  }
}

impl<'f, Fut> FromIterator<Fut> for FuturesResults<Fut::Output>
where
  Fut: Future + Send + 'f,
  Fut::Output: Send + 'static,
{
  fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = Fut>,
  {
    let (_, inner) = Scope::scope_and_block(|s: &mut TokioScope<'_, _>| {
      iter.into_iter().for_each(|fut| {
        s.spawn(fut);
      });
    });

    Self { inner }
  }
}

impl<T> Deref for FuturesResults<T> {
  type Target = FuturesResult<T>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<T> DerefMut for FuturesResults<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}
