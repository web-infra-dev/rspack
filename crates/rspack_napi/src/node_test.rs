use napi::JsUnknown;
use napi_derive::napi;

use crate::{threadsafe_function::ThreadSafeFunctionWithRef, ByRef};

#[napi]
fn eq_by_ref(obj1: ByRef<JsUnknown>, obj2: ByRef<JsUnknown>) -> bool {
  obj1 == obj2
}

#[napi]
fn tsfn_ref_call_in_main(
  tsfb_with_ref: ThreadSafeFunctionWithRef<i64, i64>,
  arg: i64,
) -> napi::Result<i64> {
  tsfb_with_ref
    .call_sync(arg)
    .map_err(|err| napi::Error::from_reason(err.to_string()))
}

#[napi]
async fn tsfn_ref_call_in_another_thread(
  tsfb_with_ref: ThreadSafeFunctionWithRef<i64, i64>,
  arg: i64,
) -> napi::Result<i64> {
  let (cx, rx) = tokio::sync::oneshot::channel::<rspack_error::Result<i64>>();
  std::thread::spawn(move || cx.send(tsfb_with_ref.call_sync(arg)).unwrap());
  rx.await
    .unwrap()
    .map_err(|err| napi::Error::from_reason(err.to_string()))
}
