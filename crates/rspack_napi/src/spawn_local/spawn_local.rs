use futures::Future;
use napi::Env;

use crate::runtime;

pub fn spawn_local<F, Fut>(env: &Env, callback: F) -> napi::Result<()>
where
  F: FnOnce(Env) -> Fut + 'static,
  Fut: Future<Output = napi::Result<()>> + 'static,
{
  let future = callback(env.to_owned());

  runtime::spawn_async_local(&env, async move {
    if let Err(error) = future.await {
      eprintln!("Uncaught Napi Error: {}", error);
    };
  })?;

  Ok(())
}
