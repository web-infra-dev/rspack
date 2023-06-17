#![feature(try_blocks)]

mod errors;
mod ext;
mod js_values;
mod utils;
pub use errors::{NapiErrorExt, NapiResultExt};

pub mod threadsafe_function;

thread_local! {
  // Safety: A single node process always share the same napi_env, so it's safe to use a thread local
  pub static NAPI_ENV: std::cell::RefCell<Option<napi::sys::napi_env>>  = Default::default();
}

pub use crate::{
  ext::{js_reg_exp_ext::JsRegExpExt, js_string_ext::JsStringExt},
  js_values::js_reg_exp::JsRegExp,
  utils::object_prototype_to_string_call,
};
