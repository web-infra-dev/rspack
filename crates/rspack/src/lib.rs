#![deny(clippy::all)]

pub mod bundle;
pub mod chunk;
pub mod external_module;

pub mod utils;
// pub mod worker;
pub mod worker2;

pub use swc_ecma_ast as ast;

pub mod plugin_driver;
pub mod statement;
pub mod structs;
pub mod mark_box;
pub mod bundler;
pub mod module_graph;
pub mod plugins;
pub mod traits;
pub mod visitors;
pub mod js_module;