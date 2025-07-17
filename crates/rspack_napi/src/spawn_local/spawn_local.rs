use futures::Future;
use napi::Env;

use crate::{runtime, ErrorCode};

pub fn spawn_local<'a, F, Fut>(env: Env, callback: F) -> napi::Result<(), ErrorCode>
where
  F: FnOnce(Env) -> Fut + 'static,
  Fut: Future<Output = napi::Result<(), ErrorCode>> + 'static,
{
  let future = callback(env);

  runtime::spawn_async_local(&env, async move { future.await })?;

  Ok(())
}
