#![feature(arbitrary_self_types, unique_rc_arc)]

pub mod css_dependency;
mod css_module;
mod parser_plugin;
pub use parser_plugin::CssExtractJsonData;
pub mod plugin;
mod runtime;
