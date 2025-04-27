use futures::Future;
use rspack_napi::napi::{
  bindgen_prelude::*, threadsafe_function::ThreadsafeFunctionCallMode, Result,
};

pub fn callbackify<R, F>(f: Function, fut: F, call_js_back: impl FnOnce() + 'static) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  let mut call_js_back = Some(Box::new(call_js_back));

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
