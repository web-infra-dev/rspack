#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(box_patterns)]
#![feature(option_get_or_insert_default)]

pub mod dependency;
mod parser_and_generator;
pub mod plugin;
pub mod runtime;
mod utils;

pub use plugin::CssPlugin;
