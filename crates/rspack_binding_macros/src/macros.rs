#[macro_export]
macro_rules! convert_napi_object_to_js_unknown {
  ($env:ident, $object:expr) => {{
    use napi::bindgen_prelude::{FromNapiValue, ToNapiValue};
    use napi::JsUnknown;

    let o = $object;
    JsUnknown::from_napi_value($env.raw(), ToNapiValue::to_napi_value($env.raw(), o)?)
  }};
}

/// Convert a raw napi pointer to a given napi value
#[macro_export]
macro_rules! convert_raw_napi_value_to_napi_value {
  ($env:ident, $ty:ty, $val:expr) => {{
    use napi::bindgen_prelude::FromNapiValue;

    <$ty>::from_napi_value($env.raw(), $val)
  }};
}

#[macro_export]
macro_rules! call_js_function_with_napi_objects {
  ($env:ident, $fn:ident, $($object:expr),*) => {{
    $fn.call(None, &[
      $(
        $crate::convert_napi_object_to_js_unknown!($env, $object)?
      ),*
    ])
  }};
}
