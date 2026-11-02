use napi::{Env, bindgen_prelude::Function};
use napi_derive::napi;

/// Internal profiling bridge. Returns false in a binding built without allocative.
#[napi(ts_args_type = "callback: (path: string) => void")]
pub fn register_allocative_snapshot_callback(
  env: Env,
  callback: Function<'static>,
) -> napi::Result<bool> {
  #[cfg(allocative)]
  {
    use std::sync::Arc;

    use napi::{JsValue, bindgen_prelude::FromNapiValue, sys};
    use rspack_napi::threadsafe_function::ThreadsafeFunction;
    use rspack_util::rspack_allocative::{FlameGraphBuilder, snapshot};

    struct CapturedBindings(FlameGraphBuilder);
    impl FromNapiValue for CapturedBindings {
      unsafe fn from_napi_value(env: sys::napi_env, _value: sys::napi_value) -> napi::Result<Self> {
        rspack_napi::allocative::capture(env).map(Self)
      }
    }
    let callback: ThreadsafeFunction<String, CapturedBindings> =
      unsafe { FromNapiValue::from_napi_value(env.raw(), callback.raw())? };
    snapshot::register(Arc::new(move |path| {
      let callback = callback.clone();
      Box::pin(async move {
        callback
          .call_with_sync(path.to_string_lossy().into_owned())
          .await
          .map(|captured| captured.0)
          .map_err(|error| error.to_string())
      })
    }))
    .map_err(napi::Error::from_reason)?;
    thread_local! {
      static CLEANUP: std::cell::RefCell<Option<napi::CleanupEnvHook<()>>> = const { std::cell::RefCell::new(None) };
    }
    let hook = match env.add_env_cleanup_hook((), |_| snapshot::unregister()) {
      Ok(hook) => hook,
      Err(error) => {
        snapshot::unregister();
        return Err(error);
      }
    };
    CLEANUP.with_borrow_mut(|cleanup| *cleanup = Some(hook));
    Ok(true)
  }
  #[cfg(not(allocative))]
  {
    let _ = (env, callback);
    Ok(false)
  }
}
