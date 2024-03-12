#![feature(try_blocks)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod callback;
pub(crate) use callback::JsCallback;

mod errors;
mod ext;
mod js_values;
mod utils;
pub use errors::{NapiErrorExt, NapiResultExt};

pub mod new_tsfn;
pub mod threadsafe_function;

thread_local! {
  // Safety: A single node process always share the same napi_env, so it's safe to use a thread local
  static NAPI_ENV: std::cell::RefCell<Option<napi::sys::napi_env>> = Default::default();
}

/// Get [napi::sys::napi_env], only intended to be called on main thread.
/// # Panic
///
/// Panics if is accessed from other thread.
pub fn get_napi_env() -> napi::sys::napi_env {
  NAPI_ENV.with(|e| e.borrow().expect("NAPI ENV should be available"))
}

/// Set [napi::sys::napi_env]
pub fn set_napi_env(napi_env: napi::sys::napi_env) {
  NAPI_ENV.with(|e| *e.borrow_mut() = Some(napi_env))
}

pub use crate::{
  ext::{js_reg_exp_ext::JsRegExpExt, js_string_ext::JsStringExt},
  js_values::js_reg_exp::JsRegExp,
  utils::downcast_into,
};
