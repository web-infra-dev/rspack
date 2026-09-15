#![feature(arbitrary_self_types, unique_rc_arc)]

mod css_syntax;
pub mod dependency;
pub mod parser_and_generator;
pub mod plugin;
pub mod runtime;
mod utils;

pub use plugin::CssPlugin;
