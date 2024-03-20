#![feature(try_blocks)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod ext;
mod js_values;
mod utils;

#[cfg(feature = "node_test")]
pub mod node_test;

mod errors;
pub use errors::{NapiErrorExt, NapiResultExt};

mod callback;
pub(crate) use callback::JsCallback;

pub mod threadsafe_function;

pub mod regexp {
  pub use crate::ext::js_reg_exp_ext::JsRegExpExt;
  pub use crate::js_values::js_reg_exp::JsRegExp;
}

pub mod string {
  pub use crate::ext::js_string_ext::JsStringExt;
}

pub use crate::utils::downcast_into;

pub mod napi {
  pub use napi::*;
}
