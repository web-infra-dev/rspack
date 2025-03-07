mod root;
pub use root::*;
pub mod object;
use napi::bindgen_prelude::{Env, Object};
mod global_scope;
pub(crate) use global_scope::*;
