#![feature(let_chains)]
#![feature(box_patterns)]
pub mod dependency;
pub(crate) mod parser_and_generator;
pub mod plugin;
mod swc_css_compiler;
mod utils;
pub mod visitors;

pub use plugin::CssPlugin;
