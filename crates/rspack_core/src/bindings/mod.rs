mod root;
use napi::bindgen_prelude::{Env, Object};
pub use root::*;
mod global_scope;
pub(crate) use global_scope::*;
mod allocator;
pub use allocator::*;
