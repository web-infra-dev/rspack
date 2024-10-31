use napi::{bindgen_prelude::FromNapiValue, JsUnknown};

pub fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> napi::Result<T> {
  <T as FromNapiValue>::from_unknown(o)
}
