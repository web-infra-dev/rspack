use futures::Future;
use rspack_napi::napi::{
  bindgen_prelude::*, threadsafe_function::ThreadsafeFunctionCallMode, Result,
};

use crate::ErrorCode;

pub fn callbackify<R, F>(
  f: Function<'static>,
  fut: F,
  call_js_back: impl FnOnce() + 'static,
) -> Result<(), ErrorCode>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R, ErrorCode>>,
{
  let mut call_js_back = Some(Box::new(call_js_back));

  let tsfn = f
    .build_threadsafe_function::<R>()
    .error_status::<ErrorCode>()
    .callee_handled::<true>()
    .max_queue_size::<1>()
    .weak::<false>()
    .build_callback(
      move |ctx: napi::threadsafe_function::ThreadsafeCallContext<_>| {
        if let Some(call_js_back) = call_js_back.take() {
          call_js_back();
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
