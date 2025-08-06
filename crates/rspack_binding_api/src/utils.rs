use futures::Future;
use rspack_napi::napi::{
  Result, bindgen_prelude::*, threadsafe_function::ThreadsafeFunctionCallMode,
};

use crate::error::ErrorCode;
/**
 *  execution workflow
 *  1. let future_result = fut.await; // get rust future result
 *  2. let f_result = f(future_result); // pass future result to js_callback
 *  3. if has rust side finalizer then call finalizer after js_callback is finished
 */
pub fn callbackify<R, F>(
  js_callback: Function<'static>,
  fut: F,
  finalizer: Option<impl FnOnce() + 'static>,
) -> Result<(), ErrorCode>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R, ErrorCode>>,
{
  let mut finalizer = finalizer.map(|x| Box::new(x));

  let tsfn = js_callback
    .build_threadsafe_function::<R>()
    .error_status::<ErrorCode>()
    .callee_handled::<true>()
    .max_queue_size::<1>()
    .weak::<false>()
    .build_callback(
      move |ctx: napi::threadsafe_function::ThreadsafeCallContext<_>| {
        if let Some(finalizer) = finalizer.take() {
          finalizer();
        }
        Ok(ctx.value)
      },
    )
    .map_err(|err| napi::Error::new(ErrorCode::Napi(err.status), err.reason.clone()))?;

  napi::bindgen_prelude::spawn(async move {
    let res = fut.await;
    tsfn.call(res, ThreadsafeFunctionCallMode::NonBlocking);
  });
  Ok(())
}
