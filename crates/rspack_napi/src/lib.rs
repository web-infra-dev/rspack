#![feature(try_blocks)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod error_code;
pub use error_code::*;
mod ext;
mod js_values;
mod runtime;
pub mod spawn_local;
mod utils;

mod errors;
pub use errors::NapiErrorToRspackErrorExt;

mod callback;
pub use callback::JsCallback;

pub mod threadsafe_function;
pub mod threadsafe_js_value_ref;

pub mod string {
  pub use crate::ext::js_string_ext::JsStringExt;
}

pub use js_values::{
  one_shot_instance_ref::*, one_shot_value_ref::*, threadsafe_one_shot_value_ref::*, value_ref::*,
  weak_ref::*,
};
pub use napi;
pub use utils::*;
