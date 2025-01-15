#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![recursion_limit = "256"]

mod plugin;
pub use crate::loader::*;
pub use crate::plugin::*;
pub use crate::utils::{
  decl::RSCAdditionalData, decl::ReactRoute, export_visitor, has_client_directive,
  has_server_directive, rsc_visitor,
};
mod loader;
mod utils;
