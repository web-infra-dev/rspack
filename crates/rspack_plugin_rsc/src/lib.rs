#![feature(option_get_or_insert_default)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![recursion_limit = "256"]

mod plugin;
pub use crate::plugin::*;
