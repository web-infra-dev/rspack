use futures::Future;
use napi::Env;

use crate::{spawn_local, ErrorCode};

pub trait SpawnLocalExt {
  /// Spawns a non-blocking future on the local thread.
  /// Normal [`NapiValue`] types can be interacted with in
  /// the async context. Supports channels, timers, etc.
  ///
  /// Equivalent to:
  ///
  /// ```javascript
  /// setTimeout(async () => { await work() }, 0)
  /// ```
  ///
  /// To ensure the availability of [`NapiValue`] types beyond the life of the parent function scope,
  /// ensure that [`NapiValue`] types that will be used in an async closure are wrapped in a [`crate::JsRc`].
  ///
  /// ### Usage:
  ///
  /// #### Running a Callback:
  ///
  /// ```
  /// use std::time::Duration;
  ///
  /// use async_std::task;
  /// use napi::*;
  /// use napi_async_local::{JsRc, JsRcExt, SpawnLocalExt};
  /// use napi_derive::napi;
  ///
  /// #[napi]
  /// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<()> {
  ///   env.spawn_local(move |env| async move {
  ///     task::sleep(Duration::from_millis(1000)).await;
  ///     callback.inner(&env)?.call_without_args(None)?;
  ///     Ok(())
  ///   })
  /// }
  /// ```
  ///
  /// #### Using Channels:
  ///
  /// ```
  /// use std::{thread, time::Duration};
  ///
  /// use async_std::channel;
  /// use napi::*;
  /// use napi_async_local::{JsRc, JsRcExt, SpawnLocalExt};
  /// use napi_derive::napi;
  ///
  /// #[napi]
  /// fn my_js_func(env: Env, callback: JsRc<JsFunction>) -> napi::Result<()> {
  ///   let (tx, rx) = channel::unbounded();
  ///
  ///   thread::spawn(move || {
  ///     for i in 0..10 {
  ///       tx.send_blocking(i).unwrap();
  ///       thread::sleep(Duration::from_millis(1000));
  ///     }
  ///   });
  ///
  ///   env.spawn_local(move |env| async move {
  ///     while let Ok(value) = rx.recv().await {
  ///       println!("Got number: {}", value);
  ///       callback
  ///         .inner(&env)?
  ///         .call(None, &[env.create_int32(value)?])?;
  ///     }
  ///
  ///     Ok(())
  ///   })
  /// }
  /// ```
  fn spawn_local<F, Fut>(&self, callback: F) -> napi::Result<(), ErrorCode>
  where
    F: FnOnce(Env) -> Fut + 'static,
    Fut: Future<Output = napi::Result<(), ErrorCode>> + 'static;
}

impl SpawnLocalExt for Env {
  fn spawn_local<F, Fut>(&self, callback: F) -> napi::Result<(), ErrorCode>
  where
    F: FnOnce(Env) -> Fut + 'static,
    Fut: Future<Output = napi::Result<(), ErrorCode>> + 'static,
  {
    spawn_local::spawn_local(*self, callback)
  }
}
