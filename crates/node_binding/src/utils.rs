use futures::Future;
use rspack_napi::napi::threadsafe_function::ThreadsafeFunctionCallMode;
use rspack_napi::napi::{bindgen_prelude::*, Result};

pub fn callbackify<R, F>(f: Function, fut: F, call_js_back: impl FnOnce() + 'static) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  // The ThreadsafeFunction can only be constructed on the JavaScript thread. The JS callback must execute on the JS thread.
  // So we can remove the Send bound for ThreadsafeFunction js call callback.
  // here: https://github.com/napi-rs/napi-rs/pull/2510
  let call_js_back = unsafe {
    std::mem::transmute::<Box<dyn FnOnce() + 'static>, Box<dyn FnOnce() + Send + Sync + 'static>>(
      Box::new(call_js_back),
    )
  };
  let mut call_js_back = Some(call_js_back);

  let tsfn = f
    .build_threadsafe_function::<Result<R>>()
    .callee_handled::<false>()
    .max_queue_size::<1>()
    .weak::<false>()
    .build_callback(
      move |_ctx: napi::threadsafe_function::ThreadsafeCallContext<_>| {
        if let Some(call_js_back) = call_js_back.take() {
          call_js_back();
        }
        Ok(())
      },
    )?;

  napi::bindgen_prelude::spawn(async move {
    let res = fut.await;
    tsfn.call(res, ThreadsafeFunctionCallMode::NonBlocking);
  });
  Ok(())
}
