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

// *Note* that the order of the creation of threadsafe function is important. There is a queue of threadsafe calls for each tsfn:
// For example:
// tsfn1: [call-in-js-task1, call-in-js-task2]
// tsfn2: [call-in-js-task3, call-in-js-task4]
// If the tsfn1 is created before tsfn2, and task1 is created(via `tsfn.call`) before task2(single tsfn level),
// and *if these tasks are created in the same tick*, tasks will be called on main thread in the order of `task1` `task2` `task3` `task4`
//
// In practice:
// The creation of callback `this_compilation` is placed before the callback `compilation` because we want the JS hooks `this_compilation` to be called before the JS hooks `compilation`.

#[macro_export]
macro_rules! js_fn_into_threadsafe_fn {
  ($js_cb:expr, $env:expr) => {{
    use napi::NapiRaw;
    use rspack_napi_shared::threadsafe_function::ThreadsafeFunction;

    let env = $env;
    let cb = unsafe { $js_cb.raw() };
    let mut tsfn = ThreadsafeFunction::create(env.raw(), cb, 0, |ctx| {
      let (ctx, resolver) = ctx.split_into_parts();

      let env = ctx.env;
      let cb = ctx.callback;
      let result = unsafe { $crate::call_js_function_with_napi_objects!(env, cb, ctx.value) };

      resolver.resolve(result, |_, v| Ok(v))
    })?;

    // See the comment in `threadsafe_function.rs`
    tsfn.unref(&env)?;
    tsfn
  }};
}
